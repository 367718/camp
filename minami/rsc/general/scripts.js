"use strict";


// -------------------- constants --------------------


const SECTION_NODE_SELECTOR = ".section";

const FILTER_NODE_SELECTOR = ".filter";
const FILTER_TIMEOUT_ATTRIBUTE = "data-timeout";
const FILTER_TIMEOUT_VALUE = 500;

const LIST_NODE_SELECTOR = ".list";
const LIST_SORTED_ATTRIBUTE = "data-sorted";
const LIST_REFRESH_ATTRIBUTE = "data-refresh";

const ENTRY_SELECTED_ATTRIBUTE = "data-selected";
const ENTRY_FILTERED_CLASS = "filtered";

const ACTIONS_NODE_SELECTOR = ".actions";
const ACTIONS_URL_ATTRIBUTE = "data-url";
const ACTIONS_CONFIRM_ATTRIBUTE = "data-confirm";
const ACTIONS_PROMPT_ATTRIBUTE = "data-prompt";
const ACTIONS_REFRESH_ATTRIBUTE = "data-refresh";

const TOGGLES_NODE_SELECTOR = ".toggles";

const HOTKEY_COPY_CONTROL = true;
const HOTKEY_COPY_COMPLETE = "KeyC";
const HOTKEY_COPY_CLEAN = "KeyX";


// -------------------- classes --------------------


class Section {
    
    constructor() {
        
        this.node = document.querySelector(SECTION_NODE_SELECTOR);
        
        if (this.node === null) {
            return;
        }
        
        this.filter = new Filter(this);
        this.list = new List(this);
        this.actions = new Actions(this);
        this.toggles = new Toggles(this);
        
        Object.freeze(this);
        
        this.node.addEventListener("mouseover", () => {
            
            // bail if filter input is involved
            if (document.activeElement === this.filter.node) {
                return;
            }
            
            this.list.node.focus();
            
        });
        
    }
    
}

class Filter {
    
    constructor(parent) {
        
        this.node = parent.node.querySelector(FILTER_NODE_SELECTOR);
        this.parent = parent;
        
        Object.freeze(this);
        
        this.node.addEventListener("input", () => {
            
            clearTimeout(this.node.getAttribute(FILTER_TIMEOUT_ATTRIBUTE));
            this.node.setAttribute(FILTER_TIMEOUT_ATTRIBUTE, setTimeout(() => this.apply(this.parent.list.entries), FILTER_TIMEOUT_VALUE));
            
        });
        
    }
    
    apply = (entries) => {
        
        entries.filter(entry => entry.is_filtered())
            .forEach(entry => entry.toggle_filter());
        
        if (this.node.value === "") {
            return;
        }
        
        const criteria = this.node.value.normalize("NFC");
        const collator = new Intl.Collator("en", { usage: "search", sensitivity: "base" });
        
        outer: for (let entry of entries) {
            
            const current = entry.text(false).normalize("NFC");
            
            for (let start = 0, end = criteria.length; end <= current.length; start++, end++) {
                if (collator.compare(criteria, current.slice(start, end)) === 0) {
                    continue outer;
                }
            }
            
            entry.toggle_filter();
            
        }
        
        entries.filter(entry => entry.is_selected() && ! entry.is_visible())
            .forEach(entry => entry.toggle_select());
        
    };
    
}

class List {
    
    constructor(parent) {
        
        this.node = parent.node.querySelector(LIST_NODE_SELECTOR);
        this.parent = parent;
        this.entries = [];
        
        // freeze would prevent the refreshing of the entries array
        Object.seal(this);
        
        this.node.addEventListener("keydown", (event) => {
            
            // bail if filter input is involved
            if (event.target === this.parent.filter.node) {
                return;
            }
            
            // copy text to clipboard
            if ((event.ctrlKey === HOTKEY_COPY_CONTROL) && (event.code === HOTKEY_COPY_COMPLETE || event.code === HOTKEY_COPY_CLEAN)) {
                this.copy(event.code === HOTKEY_COPY_CLEAN);
                return event.preventDefault();
            }
            
        });
        
        this.refresh();
        
    }
    
    toggle = (criteria) => {
        
        this.node.classList.toggle(criteria);
        
        this.entries.filter(entry => entry.is_selected() && ! entry.is_visible())
            .forEach(entry => entry.toggle_select());
        
    };
    
    select = (target, control, shift) => {
        
        // -------------------- simple click --------------------
        
        // select target and deselect every other entry
        
        if (! control && ! shift) {
            
            this.entries.filter(entry => entry.is_selected())
                .forEach(entry => entry.toggle_select());
            
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
            
            let start_index = this.entries.findIndex(entry => entry.is_selected());
            let target_index = this.entries.indexOf(target);
            
            if (start_index == -1) {
                start_index = this.entries.findIndex(entry => entry.is_visible());
            }
            
            if (start_index > target_index) {
                start_index = target_index;
                target_index = this.entries.findLastIndex(entry => entry.is_selected());
            }
            
            this.entries.filter(entry => entry.is_selected())
                .forEach(entry => entry.toggle_select());
            
            this.entries.slice(start_index, target_index + 1)
                .filter(entry => entry.is_visible())
                .forEach(entry => entry.toggle_select());
            
        }
        
    };
    
    copy = (clean) => {
        
        if (! navigator.clipboard) {
            window.alert("Access to the clipboard is only available in secure contexts or localhost")
            return;
        }
        
        const text = this.entries.filter(entry => entry.is_selected())
            .map(entry => entry.text(clean))
            .join("\n");
        
        navigator.clipboard.writeText(text);
        
    };
    
    refresh = () => {
        
        fetch(this.node.getAttribute(LIST_REFRESH_ATTRIBUTE))
            .then(response => response.text().then(text => {
                
                if (response.status != 200) {
                    this.node.replaceChildren();
                    this.entries = [];
                    window.alert(text);
                    return;
                }
                
                // children
                
                const container = document.createElement("div");
                container.innerHTML = text;
                
                const children = Array.from(container.children);
                
                if (this.node.getAttribute(LIST_SORTED_ATTRIBUTE) === "true") {
                    const collator = new Intl.Collator("en", { usage: "sort", sensitivity: "base", numeric: true });
                    children.sort((a, b) => a.children.length - b.children.length || collator.compare(a.textContent, b.textContent));
                }
                
                // entries
                
                const entries = children.map(child => new Entry(child, this));
                
                // filter
                
                this.parent.filter.apply(entries);
                
                // refresh
                
                this.node.replaceChildren(...children);
                this.entries = entries;
                
            }))
            .catch(error => window.alert(error));
        
    };
    
}

class Entry {
    
    constructor(node, parent) {
        
        this.node = node;
        this.parent = parent;
        
        Object.freeze(this);
        
        this.node.onclick = (event) => this.parent.select(this, event.ctrlKey, event.shiftKey);
        
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
        
        this.node = parent.node.querySelector(ACTIONS_NODE_SELECTOR);
        this.parent = parent;
        
        Object.freeze(this);
        
        for (const child of this.node.children) {
            child.addEventListener("click", () => {
                
                const url = child.getAttribute(ACTIONS_URL_ATTRIBUTE);
                const confirm = child.getAttribute(ACTIONS_CONFIRM_ATTRIBUTE) === "true";
                const prompt = child.getAttribute(ACTIONS_PROMPT_ATTRIBUTE) === "true";
                const refresh = child.getAttribute(ACTIONS_REFRESH_ATTRIBUTE) === "true";
                
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
        
        const form_data = new FormData();
        
        // -------------------- prompt --------------------
        
        if (prompt) {
            const input = window.prompt("The requested action requires a value");
            
            if (input === null) {
                return;
            }
            
            form_data.append("input", input);
        }
        
        // -------------------- tags --------------------
        
        this.parent.list.entries.filter(entry => entry.is_selected())
            .forEach(entry => form_data.append("tag", entry.text()));
        
        // -------------------- request --------------------
        
        fetch(url, { method: "POST", body: form_data })
            .then(response => {
                
                if (response.status != 200) {
                    response.text().then(error => window.alert(error));
                    return;
                }
                
                if (refresh) {
                    this.parent.list.refresh();
                }
                
            })
            .catch(error => window.alert(error));
        
    };
    
}

class Toggles {
    
    constructor(parent) {
        
        this.node = parent.node.querySelector(TOGGLES_NODE_SELECTOR);
        this.parent = parent;
        
        Object.freeze(this);
        
        for (const child of this.node.children) {
            
            child.addEventListener("click", (event) => this.parent.list.toggle(event.target.value));
            
            if (child.firstElementChild.checked) {
              this.parent.list.toggle(child.firstElementChild.value);
            }
            
        }
        
    }
    
}


// -------------------- initialization --------------------


document.addEventListener("DOMContentLoaded", () => new Section());
