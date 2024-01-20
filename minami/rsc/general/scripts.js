"use strict";


// -------------------- constants --------------------


const LIST_NODE_SELECTOR = ".list";
const FILTER_NODE_SELECTOR = ".panel .filter";
const TOGGLES_NODES_SELECTOR = ".panel input[type='checkbox']";
const BUTTONS_NODES_SELECTOR = ".panel a";

const LIST_SORTED_CLASS = "sorted";

const ENTRY_SELECTED_ATTRIBUTE = "data-selected";
const ENTRY_FILTERED_CLASS = "filtered";

const BUTTONS_HOTKEY_ATTRIBUTE = "data-hotkey";

const HOYKEY_COPY_CONTROL = true;
const HOYKET_COPY_COMPLETE = "KeyC";
const HOYKEY_COPY_CLEAN = "KeyX";


// -------------------- classes --------------------


class List {
    
    constructor(list_node, filter_node) {
        this.node = list_node;
        this.filter = new Filter(filter_node);
        this.entries = [];
        
        Object.seal(this);
        
        this.refresh();
    }
    
    focus = () => this.node.focus();
    
    toggle = (criteria) => {
        
        this.node.classList.toggle(criteria);
        
        this.entries.filter(entry => entry.is_selected() && ! entry.is_visible())
            .forEach(entry => entry.select());
        
    };
    
    select = (target, control, shift) => {
        
        // -------------------- simple click --------------------
        
        // select target and deselect every other entry
        
        if (! control && ! shift) {
            
            this.entries.filter(entry => entry.is_selected())
                .forEach(entry => entry.select());
            
            target.select();
            
            return;
            
        }
        
        // -------------------- control click --------------------
        
        // select target if deselected
        // deselect target if selected
        
        if (control) {
            target.select();
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
                .forEach(entry => entry.select());
            
            this.entries.slice(start_index, target_index + 1)
                .filter(entry => entry.is_visible())
                .forEach(entry => entry.select());
            
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
        
        fetch(this.node.dataset.refresh)
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
                
                if (this.node.classList.contains(LIST_SORTED_CLASS)) {
                    const collator = new Intl.Collator("en", { usage: "sort", sensitivity: "base", numeric: true });
                    children.sort((a, b) => a.children.length - b.children.length || collator.compare(a.textContent, b.textContent));
                }
                
                // entries
                
                const entries = children.map(entry_node => new Entry(entry_node));
                
                // filter
                
                this.filter.apply(entries);
                
                // refresh
                
                this.node.replaceChildren(...children);
                this.entries = entries;
                
            }))
            .catch(error => window.alert(error));
        
    };
    
}

class Filter {
    
    constructor(filter_node) {
        this.node = filter_node;
        
        this.node.addEventListener("input", () => {
            
            clearTimeout(this.node.dataset.timeout);
            this.node.dataset.timeout = setTimeout(() => this.apply(LIST.entries), 500);
            
        }, false);
        
        Object.freeze(this);
    }
    
    apply = (entries) => {
        
        entries.filter(entry => entry.is_filtered())
            .forEach(entry => entry.filter());
        
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
            
            entry.filter();
            
        }
        
        entries.filter(entry => entry.is_selected() && ! entry.is_visible())
            .forEach(entry => entry.select());
        
    };
    
}

class Entry {
    
    constructor(entry_node) {
        this.node = entry_node;
        
        this.node.onclick = (event) => LIST.select(this, event.ctrlKey, event.shiftKey);
        
        Object.freeze(this);
    }
    
    is_selected = () => this.node.hasAttribute(ENTRY_SELECTED_ATTRIBUTE);
    
    is_filtered = () => this.node.classList.contains(ENTRY_FILTERED_CLASS);
    
    is_visible = () => this.node.offsetParent != null;
    
    select = () => this.node.toggleAttribute(ENTRY_SELECTED_ATTRIBUTE);
    
    filter = () => this.node.classList.toggle(ENTRY_FILTERED_CLASS);
    
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


// -------------------- objects and events --------------------


document.addEventListener("DOMContentLoaded", () => {
    // -------------------- nodes --------------------
    
    const list_node = document.querySelector(LIST_NODE_SELECTOR);
    const filter_node = document.querySelector(FILTER_NODE_SELECTOR);
    
    if (list_node === null || filter_node === null) {
        return;
    }
    
    const toggles_nodes = Array.from(document.querySelectorAll(TOGGLES_NODES_SELECTOR));
    const buttons_nodes = Array.from(document.querySelectorAll(BUTTONS_NODES_SELECTOR));
    
    // -------------------- list --------------------
    
    Object.defineProperty(window, "LIST", {
        value: new List(list_node, filter_node),
        configurable: false,
        writable: false,
    });
    
    // -------------------- focus --------------------
    
    document.addEventListener("mouseover", (event) => {
        
        if (document.activeElement && document.activeElement.tagName === "INPUT") {
            return;
        }
        
        LIST.focus();
        
    });
    
    // -------------------- toggles --------------------
    
    toggles_nodes.forEach(toggle => toggle.addEventListener("click", (event) => LIST.toggle(event.target.value), false));
    
    // -------------------- hotkeys --------------------
    
    document.addEventListener("keydown", (event) => {
        
        // do not intercept keystrokes for filter input
        if (event.target === filter_node) {
            return;
        }
        
        // -------------------- copy text to clipboard --------------------
        
        if ((event.ctrlKey === HOYKEY_COPY_CONTROL) && (event.code === HOYKET_COPY_COMPLETE || event.code === HOYKEY_COPY_CLEAN)) {
            LIST.copy(event.code === HOYKEY_COPY_CLEAN);
            return event.preventDefault();
        }
        
        // -------------------- buttons --------------------
        
        buttons_nodes.find(button => button.getAttribute(BUTTONS_HOTKEY_ATTRIBUTE) == event.code)?.click();
        
    });
});


// -------------------- buttons --------------------


function request({ url = "", confirm = false, prompt = false, refresh = false } = {}) {
    // -------------------- confirm --------------------
    
    if (confirm && ! window.confirm("Are you sure you want to proceed with the requested action?")) {
        return;
    }
    
    // -------------------- form data --------------------
    
    const form_data = new FormData();
    
    // -------------------- prompt --------------------
    
    if (prompt) {
        const input = window.prompt("The requested action requires a value");
        
        if (input) {
            form_data.append("input", input);
        }
    }
    
    // -------------------- tags --------------------
    
    LIST.entries.filter(entry => entry.is_selected())
        .forEach(entry => form_data.append("tag", entry.text()));
    
    // -------------------- request --------------------
    
    fetch(url, { method: "POST", body: form_data })
        .then(response => {
            
            if (response.status != 200) {
                response.text().then(error => window.alert(error));
                return;
            }
            
            if (refresh) {
                LIST.refresh();
            }
            
        })
        .catch(error => window.alert(error));
}
