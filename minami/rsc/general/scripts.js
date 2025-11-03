"use strict";


// -------------------- constants --------------------


const CURRENT_NODE_SELECTOR = ".current";

const HOTKEY_COPY_CONTROL = true;
const HOTKEY_COPY_COMPLETE = "KeyC";
const HOTKEY_COPY_CLEAN = "KeyX";

const SECTIONS_NODE_SELECTOR = ".sections";
const SECTIONS_ACTIVE_CLASS = "active";

const FILTER_NODE_SELECTOR = ".filter";
const FILTER_TIMEOUT_ATTRIBUTE = "data-timeout";
const FILTER_TIMEOUT_VALUE = 500;

const LIST_NODE_SELECTOR = ".list";
const LIST_SORTED_CLASS = "sorted";
const LIST_REFRESH_ATTRIBUTE = "data-refresh";

const ENTRY_SELECTED_ATTRIBUTE = "data-selected";
const ENTRY_FILTERED_CLASS = "filtered";

const ACTIONS_NODE_SELECTOR = ".actions";
const ACTIONS_URL_ATTRIBUTE = "data-url";
const ACTIONS_CONFIRM_CLASS = "confirm";
const ACTIONS_PROMPT_CLASS = "prompt";
const ACTIONS_REFRESH_CLASS = "refresh";

const TOGGLES_NODE_SELECTOR = ".toggles";
const TOGGLES_VALUE_ATTRIBUTE = "data-value";
const TOGGLES_ACTIVE_ATTRIBUTE = "data-active";


// -------------------- classes --------------------


class Current {
  
  constructor() {
    
    // -------------------- properties --------------------
    
    this.node = document.querySelector(CURRENT_NODE_SELECTOR);
    this.sections = new Sections(this);
    this.filter = new Filter(this);
    this.list = new List(this);
    this.actions = new Actions(this);
    this.toggles = new Toggles(this);
    
    Object.freeze(this);
    
    if (this.node === null) {
      return;
    }
    
    // -------------------- bindings --------------------
    
    this.node.addEventListener("keydown", (event) => {
      
      // bail if filter input is involved
      if (this.filter.node && event.target === this.filter.node) {
        return;
      }
      
      // copy text to clipboard
      if (event.ctrlKey === HOTKEY_COPY_CONTROL && (event.code === HOTKEY_COPY_COMPLETE || event.code === HOTKEY_COPY_CLEAN)) {
        this.list?.copy(event.code === HOTKEY_COPY_CLEAN);
        return event.preventDefault();
      }
      
    });
    
  }
}

class Sections {
  
  constructor(parent) {
    
    // -------------------- properties --------------------
    
    this.node = parent.node?.querySelector(SECTIONS_NODE_SELECTOR) ?? null;
    this.parent = parent;
    
    Object.freeze(this);
    
    if (this.node === null) {
      return;
    }
    
    // -------------------- bindings --------------------
    
    let active = Array.from(this.node.children)
      .find((current) => current.classList.contains(SECTIONS_ACTIVE_CLASS));
    
    active.addEventListener("click", () => this.parent.list.refresh());
    
  }
}

class Filter {
  
  constructor(parent) {
    
    // -------------------- properties --------------------
    
    this.node = parent.node?.querySelector(FILTER_NODE_SELECTOR) ?? null;
    this.parent = parent;
    
    Object.freeze(this);
    
    if (this.node === null) {
      return;
    }
    
    // -------------------- bindings --------------------
    
    this.node.addEventListener("input", () => {
      clearTimeout(this.node.getAttribute(FILTER_TIMEOUT_ATTRIBUTE));
      this.node.setAttribute(FILTER_TIMEOUT_ATTRIBUTE, setTimeout(() => this.apply(), FILTER_TIMEOUT_VALUE));
    });
    
  }
  
  focus = () => this.node.focus();
  
  apply = () => this.parent.list.refresh();
  
}

class List {
  
  constructor(parent) {
    
    // -------------------- properties --------------------
    
    this.node = parent.node?.querySelector(LIST_NODE_SELECTOR) ?? null;
    this.parent = parent;
    this.entries = [];
    
    // freeze would prevent the refreshing of the entries array
    Object.seal(this);
    
    if (this.node === null) {
      return;
    }
    
    // -------------------- initial load --------------------
    
    this.refresh();
    
  }
  
  toggle = (criteria) => {
    
    this.node.classList.toggle(criteria);
    
    this.entries
      .filter((entry) => entry.is_selected() && ! entry.is_visible())
      .forEach((entry) => entry.toggle_select());
    
  };

  select = (target, control, shift) => {
    
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
          
          // children
          
          const container = document.createElement("div");
          container.innerHTML = text;
          
          const children = Array.from(container.children);
          
          // sort
          
          if (this.node.classList.contains(LIST_SORTED_CLASS)) {
            
            const collator = new Intl.Collator("en", {
              usage: "sort",
              sensitivity: "base",
              numeric: true,
            });
            
            children.sort((a, b) => a.children.length - b.children.length || collator.compare(a.textContent, b.textContent));
            
          }
          
          // entries
          
          const entries = children.map((child) => new Entry(child, this));
          
          // refresh
          
          this.node.replaceChildren(...children);
          this.entries = entries;
          
        });
        
      })
      .catch((error) => window.alert(error));
    
  };
  
}

class Entry {
  
  constructor(node, parent) {
    
    // -------------------- properties --------------------
    
    this.node = node;
    this.parent = parent;
    
    Object.freeze(this);
    
    if (this.node === null) {
      return;
    }
    
    // -------------------- bindings --------------------
    
    this.node.onclick = (event) => this.parent.select(this, event.ctrlKey, event.shiftKey);
    
    this.node.ontouchstart = (_event) => {
      
      let touchMove = false;
      
      this.node.ontouchmove = (_event) => (touchMove = true);
      
      this.node.ontouchend = (event) => {
        
        if (! touchMove) {
          // handle tap as "control click"
          this.parent.select(this, true, false);
        }
        
        // prevent click event from firing
        event.preventDefault();
        
      };
      
    };
    
  }
  
  is_selected = () => this.node.hasAttribute(ENTRY_SELECTED_ATTRIBUTE);
  
  is_filtered = () => this.node.classList.contains(ENTRY_FILTERED_CLASS);
  
  is_visible = () => this.node.offsetParent != null;
  
  toggle_select = () => this.node.toggleAttribute(ENTRY_SELECTED_ATTRIBUTE);
  
  toggle_filter = () => this.node.classList.toggle(ENTRY_FILTERED_CLASS);
  
  text = (clean) => {
    
    let text = this.node.textContent;
    
    if (clean) {
      
      // strip container
      text = text.replace(/^.+\\/, "");
      
      // strip format
      text = text.replace(/\.[^.]+$/, "");
      
      // strip leading square brackets and parens
      
      {
        let previous = 0;
        
        do {
          previous = text.length;
          text = text.replace(/^\[[^\]]*\]\s*|^\([^\)]*\)\s*/, "");
        } while (text.length != previous);
      }
      
      // strip trailing square brackets and parens
      
      {
        let previous = 0;
        
        do {
          previous = text.length;
          text = text.replace(/\s*\[[^\]]*\]$|\s*\([^\)]*\)$/, "");
        } while (text.length != previous);
      }
      
      // strip episode number
      text = text.replace(/\s*-\s*\d+$/, "");
      
    }
    
    return text;
    
  };
  
}

class Actions {
  
  constructor(parent) {
    
    // -------------------- properties --------------------
    
    this.node = parent.node?.querySelector(ACTIONS_NODE_SELECTOR) ?? null;
    this.parent = parent;
    
    Object.freeze(this);
    
    if (this.node === null) {
      return;
    }
    
    // -------------------- bindings --------------------
    
    for (const child of this.node.children) {
      
      child.addEventListener("click", () => {
        
        const url = child.getAttribute(ACTIONS_URL_ATTRIBUTE);
        const confirm = child.classList.contains(ACTIONS_CONFIRM_CLASS);
        const prompt = child.classList.contains(ACTIONS_PROMPT_CLASS);
        const refresh = child.classList.contains(ACTIONS_REFRESH_CLASS);
        
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
    
    this.node = parent.node?.querySelector(TOGGLES_NODE_SELECTOR) ?? null;
    this.parent = parent;
    
    Object.freeze(this);
    
    if (this.node === null) {
      return;
    }
    
    // -------------------- bindings --------------------
    
    for (const child of this.node.children) {
      
      child.addEventListener("click", (event) => {
        child.toggleAttribute(TOGGLES_ACTIVE_ATTRIBUTE);
        this.parent.list.toggle(event.target.getAttribute(TOGGLES_VALUE_ATTRIBUTE));
      });
      
    }
    
  }
  
}

// -------------------- initialization --------------------

document.addEventListener("DOMContentLoaded", () => new Current());
