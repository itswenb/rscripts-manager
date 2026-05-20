if (!customElements.get('monaco-editor')) {
class MonacoEditor extends HTMLElement {
  constructor() {
    super();
  }

  safeMakeJSON(rawValue) {
    if (typeof rawValue === "string") return rawValue;
    if (!rawValue) return "";
    try {
      return JSON.stringify(rawValue);
    } catch (error) {
      return "";
    }
  }

  safeParseJSON(rawValue) {
    if (typeof rawValue !== "string") return rawValue;
    if (!rawValue) return {};
    try {
      return JSON.parse(rawValue);
    } catch (error) {
      return {};
    }
  }

  static get observedAttributes() {
    return ["content-type", "name", "filename"];
  }

  attributeChangedCallback(name, _, newValue) {
    if (name === "content-type") this.contentType = newValue;
    if (name === "name") { this.name = newValue; this.onNameChange(); }
    if (name === "filename") this.filename = newValue;
  }

  async handleFormData(event) {
    const value = this.editor ? this.editor.getValue() : "";
    const blob = new Blob([value], { type: this.contentType });
    event.formData.append(this.name, blob, this.filename);
  }

  onNameChange() {
    this.closest("form")?.addEventListener("formdata", (event) =>
      this.handleFormData(event),
    );
  }

  async connectedCallback() {
    this.language = this.getAttribute("language") || "json";
    this.contentType = this.getAttribute("content-type");
    this.name = this.getAttribute("name");
    this.filename = this.getAttribute("filename");
    this.schema = this.safeParseJSON(this.getAttribute("schema"));
    this.defaultValue = this.safeMakeJSON(this.getAttribute("defaultvalue"));

    const container = document.createElement("div");
    container.style.cssText = "width:100%;height:100%;";
    this.appendChild(container);

    require.config({
      paths: { vs: "/static/vendor/monaco/min/vs" },
    });
    const value = this.defaultValue;
    const schema = this.schema;
    const language = this.language;
    const self = this;
    require(["vs/editor/editor.main"], function () {
      if (schema && Object.keys(schema).length) {
        monaco.languages.json.jsonDefaults.setDiagnosticsOptions({
          validate: true,
          schemas: [{ uri: "http://monaco-web-component/schema.json", fileMatch: ["*"], schema }],
        });
      }
      self.editor = monaco.editor.create(container, { value, language, automaticLayout: true });
    });
  }
}

customElements.define("monaco-editor", MonacoEditor);
}
