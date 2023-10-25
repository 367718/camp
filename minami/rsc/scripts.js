"use strict";


// -------------------- classes --------------------


class List {
    
    constructor(node) {
        this.node = node;
        
        // children
        
        const children = Array.from(node.children);
        
        if (this.node.classList.contains("sorted")) {
            const collator = new Intl.Collator("en", { usage: "sort", sensitivity: "base", numeric: true });
            children.sort((a, b) => a.children.length - b.children.length || collator.compare(a.textContent, b.textContent));
        }
        
        this.node.append(...children);
        
        // entries
        
        this.entries = children.map(child => new Entry(child));
        
        this.entries.forEach(entry => entry.node.onclick = (event) => {
            
            if (event.ctrlKey) {
                if (entry.is_selected()) {
                    this.#deselect(entry);
                } else {
                    this.#select(entry);
                }
            } else {
                this.entries.forEach(entry => entry.node.removeAttribute("data-position"))
                this.#select(entry);
            }
            
        });
        
        Object.freeze(this);
    }
    
    focus = () => this.node.focus();
    
    toggle = (criteria) => {
        
        this.node.classList.toggle(criteria);
        
        this.entries.filter(entry => ! entry.is_visible())
            .forEach(entry => this.#deselect(entry));
        
    };
    
    filter = (value) => {
        
        const criteria = value.normalize("NFC");
        const collator = new Intl.Collator("en", { usage: "search", sensitivity: "base" });
        
        outer: for (let entry of this.entries) {
            
            const current = entry.text(false).normalize("NFC");
            
            for (let start = 0, end = criteria.length; end <= current.length; start++, end++) {
                if (collator.compare(criteria, current.slice(start, end)) === 0) {
                    entry.node.classList.remove("filtered");
                    continue outer;
                }
            }
            
            /*
            
            for (let cursor = 0; cursor + criteria.length <= current.length; cursor += 1) {
                const window = current.slice(cursor, cursor + criteria.length);
                if (collator.compare(criteria, window) === 0) {
                    entry.node.classList.remove("filtered");
                    continue outer;
                }
            }
            
            */
            
            entry.node.classList.add("filtered");
            this.#deselect(entry);
            
        }
        
    };
    
    #select = (target) => {
        
        if (target.is_selected()) {
            return;
        }
        
        let position = 0;
        
        for (let entry of this.entries.filter(entry => entry.is_selected())) {
            position = Math.max(position, entry.position());
        }
        
        target.node.setAttribute("data-position", position + 1);
        
    };
    
    #deselect = (target) => {
        
        if (! target.is_selected()) {
            return;
        }
        
        const changed = target.position();
        
        for (let entry of this.entries.filter(entry => entry.is_selected())) {
            const current = entry.position();
            if (current > changed) {
                entry.node.setAttribute("data-position", current - 1);
            }
        }
        
        target.node.removeAttribute("data-position");
        
    };
    
}

class Entry {
    
    constructor(node) {
        this.node = node;
        
        Object.freeze(this);
    }
    
    is_selected = () => this.node.hasAttribute("data-position");
    
    is_visible = () => this.node.offsetParent != null;
    
    position = () => parseInt(this.node.getAttribute("data-position")) || 0;
    
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
        value: new List(document.querySelector(".list")),
        configurable: false,
        writable: false,
    });
    
    // ---------- filters ----------
    
    for (let input of document.querySelectorAll(".panel .filter")) {
        input.addEventListener("input", () => filter(input), false);
    }
    
    // ---------- toggles ----------
    
    for (let input of document.querySelectorAll(".panel input[type='checkbox']")) {
        input.addEventListener("click", (event) => LIST.toggle(event.target.value), false);
    }
    
    // ---------- focus ----------
    
    document.addEventListener("mouseover", (event) => {
        
        if (document.activeElement && document.activeElement.tagName === "INPUT") {
            return;
        }
        
        LIST.focus();
        
    });
    
    // ---------- hotkeys ----------
    
    document.addEventListener("keydown", (event) => {
        
        if (event.target.tagName === "INPUT") {
            return;
        }
        
        // ---------- buttons ----------
        
        const button = Array.from(document.querySelectorAll(".panel a"))
            .filter(button => button.hasAttribute("data-hotkey"))
            .find(button => event.code == button.getAttribute("data-hotkey"));
        
        if (button) {
            return button.click();
        }
        
        // ---------- copy text to clipboard ----------
        
        if (event.ctrlKey && (event.code === "KeyC" || event.code === "KeyX")) {
            copy(event.code === "KeyX");
            return event.preventDefault();
        }
        
    });
    
});


// -------------------- free functions --------------------


function filter(input) {
    if (input.dataset.timeout !== null) {
        clearTimeout(input.dataset.timeout);
    }
    
    input.dataset.timeout = setTimeout(() => LIST.filter(input.value), 250);
}

function copy(clean) {
    if (! navigator.clipboard) {
        window.alert("Access to the clipboard is only available in secure contexts or localhost")
        return;
    }
    
    const text = LIST.entries.filter(entry => entry.is_selected())
        .sort((first, second) => first.position() - second.position())
        .map(entry => entry.text(clean))
        .join("\n");
    
    navigator.clipboard.writeText(text);
}

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
        .sort((first, second) => first.position() - second.position())
        .forEach(entry => form_data.append("tag", entry.text()));
    
    // -------------------- request --------------------
    
    fetch(url, { method: "POST", body: form_data })
        .then(response => {
            
            if (response.status != 200) {
                response.text().then(error => window.alert(error));
                return;
            }
            
            if (refresh) {
                window.location.reload();
            }
            
        })
        .catch(error => window.alert(error));
}
