# **Architectural Blueprint for "Egui Forge": A Native Rust RAD Utility**

## **Executive Summary**

The Rust programming language, renowned for its memory safety and systems-level performance, has seen a burgeoning ecosystem of Graphical User Interface (GUI) libraries. Among these, **egui** has established itself as the premier immediate-mode library, celebrated for its portability, WebAssembly (WASM) compatibility, and ease of integration into existing game engines.1 However, the ecosystem remains critically deficient in a mature Rapid Application Development (RAD) utility—a visual interface builder comparable to Qt Designer that allows developers to construct applications graphically rather than programmatically. While nascent attempts such as egui-rad-builder exist, they often rely on rigid architectural patterns that limit extensibility and fail to deliver the polished user experience exemplified by the official egui demo.3  
This report presents a comprehensive, expert-level architectural blueprint for "Egui Forge," a proposed RAD utility designed to fill this void. Unlike previous iterations, this system is architected from the ground up to leverage a **Shadow Object Model (SOM)**—a retained-mode data structure that bridges the gap between the persistent state required by an editor and the ephemeral nature of immediate-mode rendering. The plan details a sophisticated serialization strategy utilizing serde and typetag to manage heterogeneous widget trees 5, a docking-based shell architecture inspired by egui\_dock 7, and a robust code generation pipeline leveraging the quote crate to output idiomatic, compile-ready Rust code.8  
Structured into five distinct implementation phases, this analysis provides the theoretical depth and practical roadmap required to build a tool that not only looks and feels like the native egui demo but possesses the functional power to generate production-grade applications.

## ---

**1\. Architectural Divergence and Ecosystem Analysis**

### **1.1 The Immediate Mode Paradox in Tooling**

To understand the specific challenges of building a RAD tool for egui, one must first analyze the fundamental dichotomy between the library's operational paradigm and the requirements of a visual editor. egui operates in **Immediate Mode**. In this paradigm, the user interface is defined by the code's control flow during each frame execution. Widgets are created, drawn, and discarded within a single update loop. There is no persistent "Button Object" residing in memory that allows an external editor to modify its "text" property asynchronously. The button exists only for the microsecond that the ui.button("Click Me") function is executing.9  
Conversely, a visual editor like Qt Designer 10 is inherently a **Retained Mode** application. It requires a persistent graph of objects—buttons, layouts, labels—that essentially "live" in memory. The editor needs to know that "Widget \#42" is selected, that its x-coordinate is 150 pixels, and that it is a child of "VerticalLayout \#10." This state must persist across frames to allow for modification, inspection, and serialization.  
The friction arose in early attempts like egui-rad-builder typically stems from how they bridge this gap. A common but limiting approach is to define a massive Enum containing all possible widget types. While this works for small sets, it violates the Open-Closed Principle of software design: adding a new widget requires modifying the central enum, the rendering logic, and the serialization code. This rigidity makes the tool difficult to extend and maintain, failing the requirement for a flexible, robust utility.

### **1.2 The "Egui Demo" Aesthetic and Functional Benchmark**

The user's request explicitly references the "style of the egui demo".1 This is not merely a visual preference but a functional requirement. The egui demo is characterized by:

1. **Universal Portability:** It runs identically on native desktop OSs (Linux, macOS, Windows) and inside web browsers via WebAssembly/WebGL.2  
2. **Responsive Layouts:** It utilizes dynamic resizing, collapsible side panels, and automatic layout management rather than absolute pixel positioning.  
3. **High Performance:** It targets 60 Hz refresh rates even in debug builds.2

To replicate this, "Egui Forge" must be built using eframe, the official framework that wraps egui for cross-platform deployment.1 It must avoid heavy, platform-specific dependencies that would break WASM compatibility. Furthermore, the UI itself must be "dogfooded"—the editor should be built using the very widgets it is designed to manipulate, creating a seamless visual consistency between the tool and its output.

### **1.3 The Qt Designer Functional Model**

The "Qt Builder" style implies a specific set of interaction patterns and panels that professional developers expect 10:

* **The Widget Box (Palette):** A catalog of available controls that can be dragged onto the canvas.  
* **The Form Editor (Canvas):** The central WYSIWYG region where the UI is assembled.  
* **The Object Inspector (Hierarchy):** A tree view showing the parent-child relationships of the widget graph.  
* **The Property Editor:** A panel that displays mutable attributes (properties) of the currently selected widget.  
* **Signal/Slot Editor:** A mechanism to connect events (e.g., button click) to logic (e.g., close window).

The proposed architecture maps each of these Retained Mode concepts to an Immediate Mode implementation, utilizing a **Shadow Object Model** as the intermediary.

## ---

**2\. Core System Architecture: The Shadow Object Model (SOM)**

The Shadow Object Model is the heart of Egui Forge. It is a persistent, serializable data structure that represents the state of the user's project. When the editor renders the "Canvas," it iterates over this SOM and generates the corresponding immediate-mode egui calls.

### **2.1 Polymorphism via Trait Objects**

To avoid the rigidity of Enums, the SOM utilizes Rust's trait system to achieve polymorphism. We define a core trait, WidgetNode, which acts as the contract for any element that can exist in the designer. This allows the system to be extensible; a user could theoretically load a plugin that adds a "GraphWidget" implementation of WidgetNode without recompiling the editor kernel.  
The WidgetNode trait must encapsulate three distinct behaviors:

1. **Editor Visualization:** How the widget renders itself inside the designer canvas, including drawing selection outlines or handling specific editor interactions.  
2. **Property Introspection:** How the widget exposes its configurable fields to the Property Editor panel.  
3. **Code Generation:** How the widget synthesizes the Rust code required to instantiate itself in the final application.

Crucially, because this tree needs to be saved to disk (serialized), we face a challenge: Rust's serde library does not support serialization of trait objects (Box\<dyn WidgetNode\>) out of the box because it cannot determine which concrete struct to instantiate during deserialization.

### **2.2 The Serialization Strategy: typetag**

To resolve the serialization challenge, the architecture leverages the typetag crate.6 This library provides a procedural macro that automatically generates the necessary glue code to register concrete types with a global registry. When serde encounters a Box\<dyn WidgetNode\>, typetag injects a type tag (e.g., "type": "Button") into the JSON output. Upon deserialization, it reads this tag and invokes the correct constructor.  
This approach is superior to manual enum dispatch because it decouples the serialization logic from the central registry. Each widget implementation is self-contained. If a developer adds a new DialWidget in a separate module, they simply annotate it with \#\[typetag::serde\], and it immediately becomes serializable and usable within the project file structure.6

### **2.3 The Project State Data Structure**

The root of the SOM is the ProjectState struct. This acts as the container for the entire application definition.

| Field | Type | Description |
| :---- | :---- | :---- |
| root\_node | Box\<dyn WidgetNode\> | The entry point of the UI tree, typically a Window or Panel container. |
| selection | HashSet\<Uuid\> | A set of unique identifiers for the currently selected widgets. Used to render gizmos and populate the Inspector. |
| variables | HashMap\<String, VariableDef\> | Definitions of application state variables (e.g., "counter: i32") that widgets can bind to. |
| assets | AssetManager | A registry of embedded resources like images or fonts referenced by the design. |

This structure ensures that the entire editor state is encapsulated. Operations like "Save Project" become as simple as passing this struct to serde\_json::to\_string\_pretty.

## ---

**3\. Phase 1: The Shell and Workspace Layout**

The first phase of implementation focuses on creating the "Shell"—the container application that hosts the various tools and panels. To achieve the "Qt Builder" feel, the shell must support a flexible docking system where panels can be rearranged, tabbed, or floated.

### **3.1 Docking Architecture: egui\_dock vs. egui\_tiles**

The research identifies two primary candidates for handling window management within egui: egui\_dock 7 and egui\_tiles.14  
egui\_dock mimics the behavior found in traditional IDEs like Visual Studio or Blender. It allows users to drag tabs between panes, split panes horizontally or vertically, and stack panels. It is the more mature option and aligns closely with the user's request for a "Qt Builder" style interface, which heavily relies on tabbed inspectors and palettes.  
egui\_tiles, developed by the Rerun.io team, offers a pure tiling window manager experience. While powerful and flexible, it lacks the native concept of "tabs" in the same way egui\_dock implements them, focusing more on grids of content.16  
**Decision:** The shell will be implemented using **egui\_dock**. This library provides the specific interaction patterns (tab dragging, pane resizing) that are standard in RAD tools. The shell will initialize a DockArea covering the entire viewport, with a default layout configuring the four cardinal regions: Widget Box (Left), Inspector (Right), Output (Bottom), and Canvas (Center).

### **3.2 The Tooling Panels**

#### **3.2.1 The Widget Box (Palette)**

This panel functions as the source for drag-and-drop operations. It iterates over a registry of available widget types. Unlike a static list, this registry should be dynamic, categorizing widgets into groups (e.g., "Primitives," "Layouts," "Input"). Visually, it should render small previews or icons for each widget. When a user begins dragging an item, the system must initiate a drag payload containing the *type descriptor* of the widget, not the widget instance itself.

#### **3.2.2 The Hierarchy View (Object Inspector)**

This panel provides a structured tree view of the scene graph. It is essential for selecting widgets that might be invisible or deeply nested on the canvas. The implementation will utilize a recursive tree walker that renders CollapsingHeader elements for containers and Label elements for leaf nodes.  
Crucially, the Hierarchy View must support **re-parenting via drag-and-drop**. Users should be able to drag a "Label" from one "VerticalLayout" to another within the tree. This requires integrating egui\_dnd 17 to handle the reordering of the SOM's internal vectors.

#### **3.2.3 The Inspector (Property Editor)**

The Inspector is a context-sensitive panel. It observes the global selection set from the ProjectState. If the selection is empty, it displays project-wide settings (e.g., window title, default font). If a widget is selected, it retrieves the mutable reference to that widget from the SOM and invokes its inspect() method.  
This inversion of control—where the widget defines its own inspection UI—is critical. It means the core editor does not need to know that a "Slider" has a "range" property; the Slider widget simply renders two number entry fields when asked to inspect itself. This creates a highly decoupled and maintainable codebase.

## ---

**4\. Phase 2: The Canvas and Interaction Model**

The Canvas is the most technically complex component of Egui Forge. It acts as the "Form Editor" 10, visualizing the interface as it is being built.

### **4.1 The Simulation Paradox**

The challenge here is distinguishing between *interacting with the editor* and *interacting with the widget*. If the user places a button on the canvas and clicks it, the editor should select the button, not trigger the "On Click" event of the generated app.  
To solve this, the WidgetNode::render method must support two modes: **Preview Mode** and **Edit Mode**.  
In **Edit Mode**, the render logic performs a three-step process:

1. **Render the Widget:** It draws the widget to the ui normally.  
2. **Intercept Input:** It captures the Response object returned by egui. Instead of processing the widget's internal logic, it checks the interaction state of the *editor*.  
3. **Overlay Gizmos:** If the widget is selected, the editor draws a selection rectangle (gizmo) over the widget's bounding box using the ui.painter(). This overlay acts as the visual feedback for selection.

For example, a Button widget in Edit Mode effectively becomes a "Selectable Region" that *looks* like a button. The click event is consumed by the selection manager, updating the ProjectState to mark this widget as active.

### **4.2 Handling Layouts and Containers**

Container widgets (e.g., Grid, VerticalLayout) introduce recursion. Their render method is responsible for iterating over their children and calling render on each.  
In Edit Mode, containers must also render **Drop Zones**. When the user drags a widget from the Palette over the Canvas, the container currently under the cursor must calculate the appropriate insertion point. If the user hovers between two buttons in a vertical layout, the layout must draw a horizontal line indicating where the new widget will be inserted. This requires coordinate space calculations using ui.input().pointer.hover\_pos() relative to the rect of the container's children.

### **4.3 Implementing Drag and Drop**

The drag-and-drop mechanics will leverage the egui\_dnd library 17 or the native ui.dnd\_drag\_source (introduced in recent egui versions).  
The workflow follows this sequence:

1. **Source:** The Palette detects a drag start and sets a Payload containing the WidgetType.  
2. **Target:** The Canvas (and specifically Container widgets within it) detects a Payload is hovering.  
3. **Feedback:** The Container draws a placeholder rectangle or line to visualize the drop.  
4. **Commit:** On drag\_released, the Container receives the WidgetType from the payload, instantiates a new default Box\<dyn WidgetNode\>, and inserts it into its children vector at the calculated index.

This interaction must feel fluid and "immediate," characteristic of the egui philosophy.

## ---

**5\. Phase 3: Introspection and Property Editing**

A robust RAD tool allows users to modify every aspect of a widget without touching code. Since Rust lacks runtime reflection, "Egui Forge" must implement a manual introspection system.

### **5.1 The Inspectable Pattern**

We define an interface within the WidgetNode trait specifically for property editing.

Rust

fn inspect(\&mut self, ui: \&mut Ui);

When implementing a concrete widget, such as LabelWidget, the developer implements this method to expose the relevant fields.

Rust

impl WidgetNode for LabelWidget {  
    fn inspect(\&mut self, ui: \&mut Ui) {  
        ui.heading("Label Properties");  
        ui.horizontal(|ui| {  
            ui.label("Text");  
            ui.text\_edit\_singleline(\&mut self.text);  
        });  
        ui.horizontal(|ui| {  
            ui.label("Text Color");  
            ui.color\_edit\_button\_srgba(\&mut self.color);  
        });  
    }  
}

This pattern leverages egui's immediate mode nature perfectly. As soon as the user types into the text\_edit\_singleline field in the Inspector, the self.text string is updated. Since the Canvas redraws every frame using that same self.text, the user sees the label update in real-time on the canvas. There is no need for complex "Observer" patterns or signal propagation to sync the view and the model; the immediate mode architecture ensures they are always in sync.

### **5.2 Advanced Property Types**

The Property Editor must support complex data types beyond primitives.

* **Enums:** Many egui properties are enums (e.g., Alignment::Center, Direction::TopDown). The Inspector must render these as ComboBox widgets. To avoid boilerplate, a derive macro can be created (e.g., EguiInspect) that auto-generates the ComboBox logic for any enum by iterating over its variants.18  
* **Structs:** Properties like Margin or Vec2 are structs. The Inspector should render these as collapsible groups containing sub-editors for their fields (e.g., x and y fields for a vector).  
* **Resources:** If a widget has an Image property, the Inspector shouldn't just show a text box for the path. It should provide a file picker or a dropdown of assets loaded into the ProjectState.

## ---

**6\. Phase 4: Logic, State, and Data Binding**

To fulfill the requirement of "actually creating apps," the tool must go beyond static layouts. It needs to define the application's runtime behavior. This corresponds to the "Signal/Slot Editor" in Qt.12

### **6.1 The Variable Store (App State)**

Real egui apps typically define a struct MyApp that holds the application state.20 Egui Forge must simulate this. The ProjectState will contain a definition of this virtual struct.  
Users can add variables in a dedicated "Data" panel:

* Name: counter  
* Type: i32  
* Default: 0

### **6.2 Data Binding**

In the Property Editor, any property compatible with a variable type (e.g., a Label's text string, or a Slider's float value) should offer a **Binding Option**. Instead of entering a static value, the user can select "Bind to Variable..." and choose from the Variable Store.  
In the SOM, this requires properties to be wrapped in a Bindable\<T\> enum:

Rust

enum Bindable\<T\> {  
    Static(T),  
    Bound(String), // The name of the variable in the Variable Store  
}

During code generation, a Static value generates a literal (e.g., ui.label("Hello")), while a Bound value generates a reference to the state struct (e.g., ui.label(\&self.my\_variable)).

### **6.3 The Event-Action System**

To handle logic, the SOM introduces an event system. Widgets define **Events** (e.g., Clicked, Hovered, Changed). Users can attach **Actions** to these events via the Inspector.  
Actions can be:

1. **Standard Actions:** "Increment Variable," "Set Variable," "Navigate to Screen."  
2. **Code Snippets:** A text field where the user can write raw Rust code.

For example, on a Button's Clicked event, the user might add a Code Snippet action: println\!("Button clicked\!");. This string is stored in the SOM and injected verbatim into the generated code block handling that event.

## ---

**7\. Phase 5: The Compilation Pipeline (Code Generation)**

The final phase transforms the Shadow Object Model into a standalone Rust project. This is the "Compiler" of the RAD tool.

### **7.1 The quote Strategy**

Generating code by string concatenation is error-prone and brittle. Instead, Egui Forge will utilize the quote crate.8 quote allows constructing Rust Abstract Syntax Trees (AST) using a macro that looks like Rust code.  
Each WidgetNode implements a codegen method that returns a TokenStream.

Rust

fn codegen(\&self) \-\> TokenStream {  
    let label \= \&self.text;  
    quote\! {  
        if ui.button(\#label).clicked() {  
            // Logic generated from the Event System  
        }  
    }  
}

### **7.2 Generating the Project Scaffolding**

The tool must generate more than just the UI code; it must produce a compilable project.

1. **Cargo.toml:** The tool generates the manifest file, injecting dependencies for egui, eframe, and any extra crates used by specific widgets (e.g., egui\_extras for images).  
2. **main.rs:** Generates the entry point, the main function, and the eframe::run\_native boilerplate.  
3. **app.rs:** Generates the struct MyApp definition based on the Variable Store, the impl Default for initialization, and the impl eframe::App trait.

### **7.3 The update Loop Generation**

The core update function is generated by recursively traversing the SOM.

1. **Panel Setup:** The generator first emits code for the CentralPanel, SidePanel, etc., based on the root nodes of the design.  
2. **Tree Expansion:** Inside the panel closures, it injects the TokenStream from the children widgets.  
3. **Binding Injection:** Wherever a property is bound to a variable, the generator emits self.variable\_name instead of a literal.

Finally, the generated TokenStreams are passed to a formatting utility (like prettyplease or running rustfmt on the output string) to ensure the resulting code is clean, readable, and idiomatic. This is crucial: the output should look like code a human wrote, allowing the developer to "eject" from the tool and continue development manually if desired.

## ---

**8\. Implementation Roadmap and Phased Plan**

### **Phase 1: The Kernel (Weeks 1-4)**

**Objective:** Establish the data model and serialization.

* **Task 1.1:** Define WidgetNode trait and ProjectState struct.  
* **Task 1.2:** Implement typetag serialization logic and verify JSON output.  
* **Task 1.3:** Create the "Standard Library" of widgets: Label, Button, Window, VerticalLayout, HorizontalLayout.  
* **Task 1.4:** Write unit tests to verify that a widget tree can be saved to disk and reloaded with full fidelity.

### **Phase 2: The Shell (Weeks 5-8)**

**Objective:** Create the visual workspace.

* **Task 2.1:** Initialize the eframe project targeting desktop and WASM.  
* **Task 2.2:** Integrate egui\_dock and configure the default four-pane layout.  
* **Task 2.3:** Implement the Hierarchy View using a recursive tree renderer.  
* **Task 2.4:** Build the basic Property Inspector infrastructure that responds to selection changes.

### **Phase 3: The Interactive Canvas (Weeks 9-14)**

**Objective:** Enable visual editing.

* **Task 3.1:** Implement the "Edit Mode" rendering wrapper to intercept interactions.  
* **Task 3.2:** Develop the Gizmo system for drawing selection boxes and active outlines.  
* **Task 3.3:** Integrate egui\_dnd for the Palette-to-Canvas drag-and-drop workflow.  
* **Task 3.4:** Implement drop-target logic in container widgets to calculate insertion indices.

### **Phase 4: Data & Logic (Weeks 15-20)**

**Objective:** Enable functional app design.

* **Task 4.1:** Build the Variable Store UI and the Data Binding selector in the Inspector.  
* **Task 4.2:** Implement the Event/Action editor tab in the Inspector.  
* **Task 4.3:** Add support for "Code Snippets" in the action editor.  
* **Task 4.4:** Refine the Inspector to support complex types (Colors, Vectors, Enums).

### **Phase 5: The Compiler (Weeks 21-24)**

**Objective:** Production-ready output.

* **Task 5.1:** Implement the codegen method for all standard widgets using quote.  
* **Task 5.2:** Build the project scaffolder (Cargo.toml generator).  
* **Task 5.3:** Create the "Export" modal to write files to disk.  
* **Task 5.4:** (Bonus) Add a "Live Code Preview" panel that shows the generated Rust code updating in real-time as the user edits the canvas.

## ---

**9\. Conclusion**

The development of "Egui Forge" represents a significant maturation point for the Rust GUI ecosystem. By rejecting the limitations of rigid enum-based architectures and embracing a polymorphic, serializable **Shadow Object Model**, this plan addresses the fundamental conflict between immediate-mode rendering and retained-mode editing. The proposed architecture leverages the strengths of the Rust ecosystem—serde for data persistence, typetag for dynamic typing, and quote for metaprogramming—to deliver a tool that is both powerful and idiomatic.  
This roadmap provides a clear path to creating a utility that satisfies the user's desire for the "style of the egui demo" while delivering the functional depth of "Qt Builder." By enabling developers to transition seamlessly from visual design to native Rust code, Egui Forge has the potential to become the de facto standard for rapid GUI development in Rust.

#### **Works cited**

1. egui/ARCHITECTURE.md at main · emilk/egui \- GitHub, accessed December 26, 2025, [https://github.com/emilk/egui/blob/master/ARCHITECTURE.md](https://github.com/emilk/egui/blob/master/ARCHITECTURE.md)  
2. egui: an easy-to-use GUI in pure Rust \- Crates.io, accessed December 26, 2025, [https://crates.io/crates/egui/0.9.0](https://crates.io/crates/egui/0.9.0)  
3. timschmidt/egui-rad-builder: Tool for quickly designing egui ... \- GitHub, accessed December 26, 2025, [https://github.com/timschmidt/egui-rad-builder](https://github.com/timschmidt/egui-rad-builder)  
4. egui \- Rust \- Docs.rs, accessed December 26, 2025, [https://docs.rs/egui/latest/egui/](https://docs.rs/egui/latest/egui/)  
5. SurrealDB with Egui \- Rust | SDKs | Integration, accessed December 26, 2025, [https://surrealdb.com/docs/sdk/rust/frameworks/egui](https://surrealdb.com/docs/sdk/rust/frameworks/egui)  
6. typetag \- Rust \- Docs.rs, accessed December 26, 2025, [https://docs.rs/typetag](https://docs.rs/typetag)  
7. egui\_dock — Rust GUI library // Lib.rs, accessed December 26, 2025, [https://lib.rs/crates/egui\_dock](https://lib.rs/crates/egui_dock)  
8. Rust crate for outputting Rust code? \- Rust Users Forum, accessed December 26, 2025, [https://users.rust-lang.org/t/rust-crate-for-outputting-rust-code/98724](https://users.rust-lang.org/t/rust-crate-for-outputting-rust-code/98724)  
9. Rust egui: A Step-by-Step Tutorial for Absolute Beginners \- HackMD, accessed December 26, 2025, [https://hackmd.io/@Hamze/Sys9nvF6Jl](https://hackmd.io/@Hamze/Sys9nvF6Jl)  
10. Get Your Qt Feet Wet | Mastering Qt 5 \- Packt Subscription, accessed December 26, 2025, [https://subscription.packtpub.com/book/mobile/9781788995399/1/ch01lvl1sec04/qt-designer-interface](https://subscription.packtpub.com/book/mobile/9781788995399/1/ch01lvl1sec04/qt-designer-interface)  
11. egui: an easy-to-use immediate mode GUI in Rust that runs ... \- GitHub, accessed December 26, 2025, [https://github.com/emilk/egui](https://github.com/emilk/egui)  
12. Qt 4.8: The New Qt Designer, accessed December 26, 2025, [https://tool.oschina.net/uploads/apidocs/qt/qt4-designer.html](https://tool.oschina.net/uploads/apidocs/qt/qt4-designer.html)  
13. Rust serde deserialize dynamic trait \- Stack Overflow, accessed December 26, 2025, [https://stackoverflow.com/questions/75413768/rust-serde-deserialize-dynamic-trait](https://stackoverflow.com/questions/75413768/rust-serde-deserialize-dynamic-trait)  
14. egui\_tiles \- crates.io: Rust Package Registry, accessed December 26, 2025, [https://crates.io/crates/egui\_tiles](https://crates.io/crates/egui_tiles)  
15. egui\_tiles \- Rust \- Docs.rs, accessed December 26, 2025, [https://docs.rs/egui\_tiles](https://docs.rs/egui_tiles)  
16. rerun-io/egui\_tiles: A tiling layout engine for egui with drag ... \- GitHub, accessed December 26, 2025, [https://github.com/rerun-io/egui\_tiles](https://github.com/rerun-io/egui_tiles)  
17. egui\_dnd — Rust GUI library // Lib.rs, accessed December 26, 2025, [https://lib.rs/crates/egui\_dnd](https://lib.rs/crates/egui_dnd)  
18. egui\_field\_editor \- Rust \- Docs.rs, accessed December 26, 2025, [https://docs.rs/egui\_field\_editor](https://docs.rs/egui_field_editor)  
19. egui\_field\_editor \- crates.io: Rust Package Registry, accessed December 26, 2025, [https://crates.io/crates/egui\_field\_editor/0.2.0](https://crates.io/crates/egui_field_editor/0.2.0)  
20. How to access egui/eframe values from other widgets?, accessed December 26, 2025, [https://stackoverflow.com/questions/74492580/how-to-access-egui-eframe-values-from-other-widgets](https://stackoverflow.com/questions/74492580/how-to-access-egui-eframe-values-from-other-widgets)