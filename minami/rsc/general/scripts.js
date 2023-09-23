"use strict";


// -------------------- classes --------------------


class List {
    
    constructor(node) {
        this.node = node;
        this.entries = Array.from(node.children).map(child => new Entry(child));
        Object.freeze(this);
    }
    
    onkeydown = (fn) => this.node.addEventListener("keydown", fn, false);
    
    entry = (ext) => this.entries.find(entry => entry.node.isEqualNode(ext));
    toggle = (criteria) => this.node.classList.toggle(criteria);
    
    clear_selection = () => this.entries.forEach(entry => entry.node.removeAttribute("data-position"));
    
    select = (target) => {
        
        let position = 0;
        
        for (let entry of this.entries.filter(entry => entry.is_selected())) {
            position = Math.max(position, entry.position());
        }
        
        target.node.setAttribute("data-position", position + 1);
        
    };
    
    deselect = (target) => {
        
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
    
    click = () => this.node.click();
    onclick = (fn) => this.node.addEventListener("click", fn, false);
    
    filter = () => this.node.classList.add("filtered");
    unfilter = () => this.node.classList.remove("filtered");
    
    is_selected = () => this.node.hasAttribute("data-position");
    is_visible = () => this.node.offsetParent != null;
    
    text = () => this.node.textContent;
    position = () => parseInt(this.node.getAttribute("data-position")) || 0;
    
}


// -------------------- events --------------------


document.addEventListener("DOMContentLoaded", () => {
    
    Object.defineProperty(window, "LIST", {
        value: new List(document.querySelector(".list")),
        configurable: false,
        writable: false,
    });
    
    document.querySelector(".filter").addEventListener("input", () => filter(), false);
    
    for (let input of document.querySelectorAll(".panel input[type='checkbox']")) {
        input.addEventListener("click", (event) => toggle(event.target.value), false);
    }
    
    for (let button of document.querySelectorAll(".panel button")) {
        if (hotkey = button.getAttribute("data-hotkey")) {
            LIST.onkeydown((event) => {
                
                if (event.code === hotkey) {
                    button.click();
                }
                
            });
        }
    }
    
    LIST.entries.forEach(entry => entry.onclick((event) => select(entry, event.ctrlKey)));
    
    LIST.onkeydown((event) => {
        
        // ---------- clear selection ----------
        
        if (event.code === "Escape") {
            clear();
            return event.preventDefault();
        }
        
        // ---------- copy names to clipboard ----------
        
        if (event.ctrlKey && event.code === "KeyC") {
            copy();
            return event.preventDefault();
        }
        
    });
    
});


// -------------------- functionality --------------------


function filter() {
    const input = document.querySelector(".filter");
    
    if (input.dataset.timeout !== null) {
        clearTimeout(input.dataset.timeout);
    }
    
    input.dataset.timeout = setTimeout(() => {
        
        const regex = new RegExp(input.value, "i");
        
        for (let entry of LIST.entries) {
            
            if (regex.exec(entry.text())) {
                
                entry.unfilter();
                
            } else {
                
                entry.filter();
                
                if (entry.is_selected()) {
                    LIST.deselect(entry);
                }
                
            }
            
        }
        
    }, 250);
}

function toggle(criteria) {
    LIST.toggle(criteria);
    
    LIST.entries.filter(entry => entry.is_selected() && ! entry.is_visible())
        .forEach(entry => LIST.deselect(entry));
}

function select(target, multiple) {
    if (multiple) {
        
        if (target.is_selected()) {
            LIST.deselect(target);
        } else {
            LIST.select(target);
        }
        
    } else {
        
        LIST.clear_selection();
        LIST.select(target);
        
    }
}

function clear() {
    LIST.clear_selection();
}

function copy() {
    if (! navigator.clipboard) {
        window.alert("Access to the clipboard is only available in secure contexts or localhost")
        return;
    }
    
    const text = LIST.entries.filter(entry => entry.is_selected())
        .sort((first, second) => first.position() - second.position())
        .map(entry => entry.text())
        .join("\n");
    
    navigator.clipboard.writeText(text);
}

function request({ url = "", prompt = false, refresh = false } = {}) {
    const form_data = new FormData();
    
    LIST.entries.filter(entry => entry.is_selected())
        .sort((first, second) => first.position() - second.position())
        .forEach(entry => form_data.append("tag", entry.text()));
    
    if (prompt) {
        const input = window.prompt("The requested action requires a value");
        
        if (! input) {
            return;
        }
        
        form_data.append("input", input);
    }
    
    fetch(url, { method: "POST", body: form_data })
        .then(response => {
            
            if (response.status == 200) {
                if (refresh) {
                    window.location.reload();
                }
            } else {
                response.text().then(error => window.alert(error));
            }
            
        })
        .catch(error => window.alert(error));
}
