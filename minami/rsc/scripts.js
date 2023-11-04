"use strict";


// -------------------- classes --------------------


class List {
    
    constructor() {
        this.node = document.querySelector(".list");
        this.filter = new Filter();
        this.entries = [];
        
        this.refresh();
        
        Object.seal(this);
    }
    
    focus = () => this.node.focus();
    
    toggle = (criteria) => {
        
        this.node.classList.toggle(criteria);
        
        this.entries.filter(entry => ! entry.is_visible())
            .forEach(entry => this.deselect(entry));
        
    };
    
    select = (target, control, shift) => {
        
        // -------------------- simple click --------------------
        
        // select target and deselect every other entry
        
        if (! control && ! shift) {
            this.entries.forEach(entry => entry.node.removeAttribute("data-position"));
            target.node.setAttribute("data-position", 1);
            return;
        }
        
        // -------------------- control click --------------------
        
        // deselect target if selected
        // select target if deselected
        
        if (control) {
            
            if (target.is_selected()) {
                
                const changed = target.position();
                
                for (let entry of this.entries.filter(entry => entry.is_selected())) {
                    const current = entry.position();
                    if (current > changed) {
                        entry.node.setAttribute("data-position", current - 1);
                    }
                }
                
                target.node.removeAttribute("data-position");
                
            } else {
                
                let position = 0;
                
                for (let entry of this.entries.filter(entry => entry.is_selected())) {
                    position = Math.max(position, entry.position());
                }
                
                target.node.setAttribute("data-position", position + 1);
                
            }
            
            return;
            
        }
        
        // -------------------- shift click --------------------
        
        // keep last selected and select every entry up to and including target
        // does nothing if last selected and target are the same entry
        
        if (shift) {
            
            const start = this.entries.filter(entry => entry.is_selected())
                .reduce((a, b) => a.position() > b.position() ? a : b);
            
            if (! start || start == target) {
                return;
            }
            
            this.entries.forEach(entry => entry.node.removeAttribute("data-position"));
            start.node.setAttribute("data-position", 1);
            
            let entries = this.entries.filter(entry => entry.is_visible());
            
            if (entries.indexOf(start) > entries.indexOf(target)) {
                entries.reverse();
            }
            
            let position = 2;
            
            for (const current of entries.slice(entries.indexOf(start) + 1)) {
                
                current.node.setAttribute("data-position", position);
                position++;
                
                if (current == target) {
                    break;
                }
                
            }
            
        }
        
    };
    
    copy = (clean) => {
        
        if (! navigator.clipboard) {
            window.alert("Access to the clipboard is only available in secure contexts or localhost")
            return;
        }
        
        const text = this.entries.filter(entry => entry.is_selected())
            .sort((first, second) => first.position() - second.position())
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
                
                // container
                
                const container = document.createElement("div");
                
                container.innerHTML = text;
                
                // children
                
                const children = Array.from(container.children);
                
                if (this.node.classList.contains("sorted")) {
                    const collator = new Intl.Collator("en", { usage: "sort", sensitivity: "base", numeric: true });
                    children.sort((a, b) => a.children.length - b.children.length || collator.compare(a.textContent, b.textContent));
                }
                
                this.node.replaceChildren(...children);
                
                // entries
                
                this.entries = children.map(child => new Entry(child));
                
                // filter
                
                this.filter.apply();
                
            }))
            .catch(error => window.alert(error));
        
    };
    
}

class Filter {
    
    constructor() {
        this.node = document.querySelector(".panel .filter");
        
        this.node.addEventListener("input", () => {
            
            clearTimeout(this.node.dataset.timeout);
            this.node.dataset.timeout = setTimeout(() => apply(), 250);
            
        }, false);
        
        Object.freeze(this);
    }
    
    apply = () => {
        
        if (this.node.value === "") {
            return;
        }
        
        const criteria = this.node.value.normalize("NFC");
        const collator = new Intl.Collator("en", { usage: "search", sensitivity: "base" });
        
        outer: for (let entry of LIST.entries) {
            
            const current = entry.text(false).normalize("NFC");
            
            for (let start = 0, end = criteria.length; end <= current.length; start++, end++) {
                if (collator.compare(criteria, current.slice(start, end)) === 0) {
                    entry.node.classList.remove("filtered");
                    continue outer;
                }
            }
            
            entry.node.classList.add("filtered");
            LIST.deselect(entry);
            
        }
        
    };
    
}

class Entry {
    
    constructor(node) {
        this.node = node;
        
        this.node.onclick = (event) => LIST.select(this, event.ctrlKey, event.shiftKey);
        
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
        value: new List(),
        configurable: false,
        writable: false,
    });
    
    // ---------- focus ----------
    
    document.addEventListener("mouseover", (event) => {
        
        if (document.activeElement && document.activeElement.tagName === "INPUT") {
            return;
        }
        
        LIST.focus();
        
    });
    
    // ---------- toggles ----------
    
    for (let input of document.querySelectorAll(".panel input[type='checkbox']")) {
        input.addEventListener("click", (event) => LIST.toggle(event.target.value), false);
    }
    
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
                LIST.refresh();
            }
            
        })
        .catch(error => window.alert(error));
}
