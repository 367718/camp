"use strict";


// -------------------- constants --------------------


const CURRENT_SECTION_NODE_SELECTOR = ".current-section";
const CURRENT_SECTION_BUTTON_SELECTOR = ".sections a:not([href])";

const HOTKEY_COPY_COMPLETE = "KeyC";
const HOTKEY_COPY_CLEAN = "KeyX";

const FILTER_NODE_SELECTOR = ".filter";
const FILTER_TIMEOUT_ATTRIBUTE = "data-timeout";
const FILTER_TIMEOUT_VALUE = 500;

const LIST_NODE_SELECTOR = ".list";
const LIST_SORTED_ATTRIBUTE = "data-sorted";
const LIST_REFRESH_ATTRIBUTE = "data-refresh";

const ENTRY_SELECTED_ATTRIBUTE = "data-selected";

const ACTIONS_NODE_SELECTOR = ".actions";
const ACTIONS_URL_ATTRIBUTE = "data-url";
const ACTIONS_CONFIRM_ATTRIBUTE = "data-confirm";
const ACTIONS_PROMPT_ATTRIBUTE = "data-prompt";
const ACTIONS_REFRESH_ATTRIBUTE = "data-refresh";

const TOGGLES_NODE_SELECTOR = ".toggles";
const TOGGLES_ATTR_ATTRIBUTE = "data-attr";
const TOGGLES_ENABLED_ATTRIBUTE = "data-enabled";

const COLOR_CLASSES = ["rin", "nadeshiko", "aoi", "chiaki", "ena"];


// -------------------- classes --------------------


class CurrentSection {
  
  constructor() {
    
    // -------------------- properties --------------------
    
    this.node = document.querySelector(CURRENT_SECTION_NODE_SELECTOR);
    
    if (this.node === null) {
      return;
    }
    
    this.filter = new Filter(this);
    this.list = new List(this);
    this.actions = new Actions(this);
    this.toggles = new Toggles(this);
    
    // -------------------- styles --------------------
    
    const color = COLOR_CLASSES[Math.floor(Math.random() * COLOR_CLASSES.length)];
    this.node.classList.add(color);
    
    // -------------------- bindings --------------------
    
    // refresh list on current section button click
    const button = this.node.querySelector(CURRENT_SECTION_BUTTON_SELECTOR);
    button.addEventListener("click", () => this.list.refresh());
    
    this.node.addEventListener("keydown", (event) => {
      
      // bail if filter input is involved
      if (this.filter.node && event.target === this.filter.node) {
        return;
      }
      
      // copy text to clipboard
      if (event.ctrlKey && (event.code === HOTKEY_COPY_COMPLETE || event.code === HOTKEY_COPY_CLEAN)) {
        this.list.copy(event.code === HOTKEY_COPY_CLEAN);
        return event.preventDefault();
      }
      
    });
    
    // -------------------- initial load --------------------
    
    this.list.refresh();
    
  }
  
}

class Filter {
  
  constructor(parent) {
    
    // -------------------- properties --------------------
    
    this.node = parent.node.querySelector(FILTER_NODE_SELECTOR);
    this.parent = parent;
    
    // -------------------- bindings --------------------
    
    this.node.addEventListener("input", () => {
      clearTimeout(this.node.getAttribute(FILTER_TIMEOUT_ATTRIBUTE));
      const timeout = setTimeout(() => this.parent.list.refresh(), FILTER_TIMEOUT_VALUE);
      this.node.setAttribute(FILTER_TIMEOUT_ATTRIBUTE, timeout);
    });
    
  }
  
}

class List {
  
  constructor(parent) {
    
    // -------------------- properties --------------------
    
    this.node = parent.node.querySelector(LIST_NODE_SELECTOR);
    this.parent = parent;
    this.entries = [];
    this.parser = new DOMParser();
    this.collator = new Intl.Collator("en", { numeric: true });
    
    // -------------------- bindings --------------------
    
    this.node.onclick = (event) => this.select(event.target, event.ctrlKey, event.shiftKey);
    
  }
  
  select = (node, control, shift) => {
    
    let target_node = node;
    
    // container element
    if (target_node.tagName === 'SPAN') {
      target_node = target_node.parentNode;
    }
    
    let target = this.entries.find((entry) => entry.node == target_node);
    
    if (! target) {
      return;
    }
    
    // -------------------- simple click --------------------
    
    // select target and deselect every other entry
    
    if (! control && ! shift) {
      
      this.entries
        .filter((entry) => entry.is_selected())
        .forEach((entry) => entry.toggle_select());
      
      target.toggle_select();
      
      return;
      
    }
    
    // -------------------- control click --------------------
    
    // select target if deselected
    // deselect target if selected
    
    if (control) {
      target.toggle_select();
      return;
    }
    
    // -------------------- shift click --------------------
    
    // if target is positioned after the first selected entry, make a new selection from the first selected entry up to target
    // if target is positioned before the first selected entry, make a new selection from the target up to last selected entry
    // if no entry is selected, make a new selection from the first visible entry up to target
    
    if (shift) {
      
      let start = this.entries.findIndex((entry) => entry.is_selected());
      let end = this.entries.indexOf(target);
      
      if (start == -1) {
        start = this.entries.findIndex((entry) => entry.is_visible());
      }
      
      if (start > end) {
        start = end;
        end = this.entries.findLastIndex((entry) => entry.is_selected());
      }
      
      this.entries
        .filter((entry) => entry.is_selected())
        .forEach((entry) => entry.toggle_select());
      
      this.entries
        .slice(start, end + 1)
        .filter((entry) => entry.is_visible())
        .forEach((entry) => entry.toggle_select());
      
    }
    
  };
  
  copy = (clean) => {
    
    if (! navigator.clipboard) {
      window.alert("Access to the clipboard is only available in secure contexts or localhost");
      return;
    }
    
    const text = this.entries
      .filter((entry) => entry.is_selected())
      .map((entry) => entry.text(clean))
      .join("\n");
    
    navigator.clipboard.writeText(text);
    
  };
  
  refresh = () => {
    
    const resource = this.node.getAttribute(LIST_REFRESH_ATTRIBUTE);
    const base = window.location.origin;
    const filter = this.parent.filter.node.value;
    
    const url = new URL(resource, base);
    url.searchParams.set("filter", filter);
    
    fetch(url)
      .then((response) => {
        
        if (response.status != 200) {
          this.node.replaceChildren();
          this.entries = [];
          response.text().then((error) => window.alert(error));
          return;
        }
        
        response.text().then((text) => {
          
          // parse
          
          const parsed = this.parser.parseFromString(text, "text/html");
          const children = Array.from(parsed.body.childNodes);
          
          // sort
          
          if (this.node.getAttribute(LIST_SORTED_ATTRIBUTE) == "true") {
            children.sort((a, b) => a.children.length - b.children.length || this.collator.compare(a.textContent, b.textContent));
          }
          
          // update
          
          this.node.replaceChildren(...children);
          this.entries = children.map((child) => new Entry(child));
          
        });
        
      })
      .catch((error) => window.alert(error));
    
  };
  
}

class Actions {
  
  constructor(parent) {
    
    // -------------------- properties --------------------
    
    this.node = parent.node.querySelector(ACTIONS_NODE_SELECTOR);
    this.parent = parent;
    
    // -------------------- bindings --------------------
    
    for (const child of this.node.children) {
      
      child.addEventListener("click", () => {
        
        const url = child.getAttribute(ACTIONS_URL_ATTRIBUTE);
        const confirm = child.getAttribute(ACTIONS_CONFIRM_ATTRIBUTE) == "true";
        const prompt = child.getAttribute(ACTIONS_PROMPT_ATTRIBUTE) == "true";
        const refresh = child.getAttribute(ACTIONS_REFRESH_ATTRIBUTE) == "true";
        
        this.request(url, confirm, prompt, refresh);
        
      });
      
    }
    
  }
  
  request = (url, confirm, prompt, refresh) => {
    
    // -------------------- confirm --------------------
    
    if (confirm && ! window.confirm("Are you sure you want to proceed with the requested action?")) {
      return;
    }
    
    // -------------------- form data --------------------
    
    const form_data = new URLSearchParams();
    
    // -------------------- prompt --------------------
    
    if (prompt) {
      
      const input = window.prompt("The requested action requires a value");
      
      if (input === null) {
        return;
      }
      
      form_data.append("input", input);
      
    }
    
    // -------------------- matcher --------------------
    
    this.parent.list.entries
      .filter((entry) => entry.is_selected())
      .forEach((entry) => form_data.append("matcher", entry.text()));
    
    // -------------------- request --------------------
    
    fetch(url, { method: "POST", body: form_data })
      .then((response) => {
        
        if (response.status != 200) {
          response.text().then((error) => window.alert(error));
          return;
        }
        
        if (refresh) {
          this.parent.list.refresh();
        }
        
      })
      .catch((error) => window.alert(error));
    
  };
  
}

class Toggles {
  
  constructor(parent) {
    
    // -------------------- properties --------------------
    
    this.node = parent.node.querySelector(TOGGLES_NODE_SELECTOR);
    this.parent = parent;
    
    // -------------------- bindings --------------------
    
    for (const child of this.node.children) {
      
      child.addEventListener("click", (event) => {
        
        const state = child.getAttribute(TOGGLES_ENABLED_ATTRIBUTE) == "true" ? "false" : "true";
        const attr = child.getAttribute(TOGGLES_ATTR_ATTRIBUTE);
        
        child.setAttribute(TOGGLES_ENABLED_ATTRIBUTE, state);
        
        this.parent.list.node.setAttribute(attr, state);
        
        this.parent.list.entries
          .filter((entry) => entry.is_selected() && ! entry.is_visible())
          .forEach((entry) => entry.toggle_select());
        
      });
      
    }
    
  }
  
}

class Entry {
  
  constructor(node) {
    
    this.node = node;
    
  }
  
  is_selected = () => this.node.hasAttribute(ENTRY_SELECTED_ATTRIBUTE);
  
  is_visible = () => this.node.offsetParent != null;
  
  toggle_select = () => this.node.toggleAttribute(ENTRY_SELECTED_ATTRIBUTE);
  
  text = (clean) => {
    
    let text = this.node.textContent;
    
    if (clean) {
      
      // container
      text = text.replace(/^.+\\/, "");
      
      // format
      text = text.replace(/\.[^.]+$/, "");
      
      // square brackets and parens
      text = text.replace(/\[[^\]]*\]\s*|\([^\)]*\)\s*/g, "");
      
      // episode number
      text = text.replace(/\s*-*\s*\d+\s*$/, "");
      
    }
    
    return text;
    
  };
  
}


// -------------------- initialization --------------------


document.addEventListener("DOMContentLoaded", () => new CurrentSection());
