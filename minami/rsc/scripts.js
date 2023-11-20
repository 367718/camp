"use strict";


// -------------------- classes --------------------


class List {
    
    constructor() {
        this.node = document.querySelector(".list");
        this.filter = new Filter();
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
                
                if (this.node.classList.contains("sorted")) {
                    const collator = new Intl.Collator("en", { usage: "sort", sensitivity: "base", numeric: true });
                    children.sort((a, b) => a.children.length - b.children.length || collator.compare(a.textContent, b.textContent));
                }
                
                // entries
                
                const entries = children.map(child => new Entry(child));
                
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
    
    constructor() {
        this.node = document.querySelector(".panel .filter");
        
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
    
    constructor(node) {
        this.node = node;
        
        this.node.onclick = (event) => LIST.select(this, event.ctrlKey, event.shiftKey);
        
        Object.freeze(this);
    }
    
    is_selected = () => this.node.hasAttribute("data-selected");
    
    is_filtered = () => this.node.classList.contains("filtered");
    
    is_visible = () => this.node.offsetParent != null;
    
    select = () => this.node.toggleAttribute("data-selected");
    
    filter = () => this.node.classList.toggle("filtered");
    
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


// -------------------- init --------------------


document.addEventListener("DOMContentLoaded", () => {
    
    Object.defineProperty(window, "LIST", {
        value: new List(),
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
    
    for (let input of document.querySelectorAll(".panel input[type='checkbox']")) {
        input.addEventListener("click", (event) => LIST.toggle(event.target.value), false);
    }
    
    // -------------------- hotkeys --------------------
    
    document.addEventListener("keydown", (event) => {
        
        if (event.target.tagName === "INPUT") {
            return;
        }
        
        // -------------------- buttons --------------------
        
        const button = Array.from(document.querySelectorAll(".panel a"))
            .filter(button => button.hasAttribute("data-hotkey"))
            .find(button => event.code == button.getAttribute("data-hotkey"));
        
        if (button) {
            return button.click();
        }
        
        // -------------------- copy text to clipboard --------------------
        
        if (event.ctrlKey && (event.code === "KeyC" || event.code === "KeyX")) {
            LIST.copy(event.code === "KeyX");
            return event.preventDefault();
        }
        
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
        
        if (! input) {
            return;
        }
        
        form_data.append("input", input);
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
