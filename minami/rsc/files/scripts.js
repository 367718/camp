// -------------------- remote resources --------------------


const PLAY_URL = "/files/play";
const MARK_URL = "/files/mark";
const MOVE_URL = "/files/move";
const LOOKUP_URL = "/files/lookup";


// -------------------- hotkeys --------------------


const SELECT_ACTIVATE_HOTKEY = (event) => ["KeyD", "ArrowRight"].includes(event.code);
const SELECT_DEACTIVATE_HOTKEY = (event) => ["KeyA", "ArrowLeft"].includes(event.code);
const SELECT_CLEAR_HOTKEY = (event) => event.code === "KeyF" && ! event.ctrlKey;

const FOCUS_UP_HOTKEY = (event) => ["KeyW", "ArrowUp"].includes(event.code);
const FOCUS_DOWN_HOTKEY = (event) => ["KeyS", "ArrowDown"].includes(event.code);
const FOCUS_JUMP_UP_HOTKEY = (event) => event.code === "PageUp";
const FOCUS_JUMP_DOWN_HOTKEY = (event) => event.code === "PageDown";

const PLAY_HOTKEY = (event) => ["Enter", "NumpadEnter", "KeyE"].includes(event.code);
const MARK_HOTKEY = (event) => ["Delete", "KeyR"].includes(event.code);
const COPY_HOTKEY = (event) => event.code === "KeyC";


// -------------------- enums --------------------


const FocusDistance = {
    Normal: 1,
    Extended: 10,
}

const FocusDirection = {
    Up: -1,
    Down: 1,
}

const SelectMethod = {
    Single: Symbol(0),
    Multiple: Symbol(0),
}

const SelectAction = {
    Toggle: Symbol(0),
    Activate: Symbol(0),
    Deactivate: Symbol(0),
    Clear: Symbol(0),
}


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
    
}

class Entry {
    
    constructor(node) {
        this.node = node;
        Object.freeze(this);
    }
    
    onclick = (fn) => this.node.addEventListener("click", fn, false);
    
    focus = () => this.node.focus();
    
    filter = () => this.node.classList.add("filtered");
    unfilter = () => this.node.classList.remove("filtered");
    
    select = (position) => this.node.setAttribute("data-position", parseInt(position) || 0);
    deselect = () => this.node.removeAttribute("data-position");
    
    is_selected = () => this.node.hasAttribute("data-position");
    is_visible = () => this.node.offsetParent != null;
    
    position = () => parseInt(this.node.getAttribute("data-position")) || 0;
    path = () => this.node.textContent;
    
}


// -------------------- event binding --------------------


document.addEventListener("DOMContentLoaded", () => {
    
    Object.defineProperty(window, "LIST", {
        value: new List(document.querySelector(".list")),
        configurable: false,
        writable: false,
    });
    
    document.querySelector(".filter").addEventListener("input", () => filter(), false);
    
    for (input of document.querySelectorAll(".panel input[type='checkbox']")) {
        input.addEventListener("click", (event) => toggle(event.target.value), false);
    }
    
    LIST.entries.forEach(entry => entry.onclick((event) => {
        
        if (event.ctrlKey) {
            select(entry, SelectMethod.Multiple, SelectAction.Activate);
        } else {
            select(entry, SelectMethod.Single, SelectAction.Activate);
        }
        
    }));
    
    LIST.onkeydown((event) => {
        
        // ---------- select entry ----------
        
        // activate
        
        if (SELECT_ACTIVATE_HOTKEY(event)) {
            select(LIST.entry(event.target), SelectMethod.Multiple, SelectAction.Activate);
            return event.preventDefault();
        }
        
        // deactivate
        
        if (SELECT_DEACTIVATE_HOTKEY(event)) {
            select(LIST.entry(event.target), SelectMethod.Multiple, SelectAction.Deactivate);
            return event.preventDefault();
        }
        
        // clear
        
        if (SELECT_CLEAR_HOTKEY(event)) {
            select(null, SelectMethod.Multiple, SelectAction.Clear);
            return event.preventDefault();
        }
        
        // ---------- focus entry ----------
        
        // up
        
        if (FOCUS_UP_HOTKEY(event)) {
            focus(FocusDistance.Normal, FocusDirection.Up);
            return event.preventDefault();
        }
        
        // down
        
        if (FOCUS_DOWN_HOTKEY(event)) {
            focus(FocusDistance.Normal, FocusDirection.Down);
            return event.preventDefault();
        }
        
        // jump up
        
        if (FOCUS_JUMP_UP_HOTKEY(event)) {
            focus(FocusDistance.Extended, FocusDirection.Up);
            return event.preventDefault();
        }
        
        // jump down
        
        if (FOCUS_JUMP_DOWN_HOTKEY(event)) {
            focus(FocusDistance.Extended, FocusDirection.Down);
            return event.preventDefault();
        }
        
        // ---------- play selected files ----------
        
        if (PLAY_HOTKEY(event)) {
            play();
            return event.preventDefault();
        }
        
        // ---------- mark selected files ----------
        
        if (MARK_HOTKEY(event)) {
            mark();
            return event.preventDefault();
        }
        
        // ---------- copy names to clipboard ----------
        
        if (COPY_HOTKEY(event)) {
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
        
        for (entry of LIST.entries) {
            
            if (regex.exec(entry.path())) {
                entry.unfilter();
            } else {
                entry.filter();
                select(entry, SelectMethod.Keyboard, SelectAction.Deactivate);
            }
            
        }
        
    }, 250);
}

function toggle(criteria) {
    LIST.toggle(criteria);
    
    if (entry = LIST.entries.find(entry => entry.is_selected() && ! entry.is_visible())) {
        entry.click();
    }
}

function select(target, method, action) {
    if (method == SelectMethod.Single) {
        
        // disallow multiple selection by deselecting everything first
        LIST.entries.filter(entry => entry.is_selected())
            .forEach(entry => entry.deselect());
        
        target.select(1);
        
    } else {
        
        // deselect every entry
        
        if (action == SelectAction.Clear) {
            LIST.entries.filter(entry => entry.is_selected())
                .forEach(entry => entry.deselect());
            return;
        }
        
        // bail if state needs no change
        
        if ((action == SelectAction.Activate && target.is_selected()) || (action == SelectAction.Deactivate && ! target.is_selected())) {
            return;
        }
        
        // if entry was unselected, re-calculate every higher position
        // if entry was selected, calculate next position
        
        const selected = LIST.entries.filter(entry => entry.is_selected());
        
        if (target.is_selected()) {
            
            const changed = target.position();
            
            for (entry of selected) {
                const current = entry.position();
                if (current > changed) {
                    entry.select(current - 1);
                }
            }
            
            target.deselect();
            
        } else {
            
            let position = 0;
            
            for (entry of selected) {
                position = Math.max(position, entry.position());
            }
            
            target.select(position + 1);
            
        }
        
    }
    
    target.focus();
}

function focus(distance, direction) {
    const current = LIST.entry(document.activeElement);
    
    if (current) {
        
        const visible = LIST.entries.filter(entry => entry.is_visible());
        
        const change = visible.indexOf(current) + distance * direction;
        const index = Math.min(Math.max(change, 0), visible.length - 1);
        
        visible[index].focus();
        
    } else {
        
        const first = LIST.entries.find(entry => entry.is_visible());
        
        if (first) {
            first.focus();
        }
        
    }
}

function play() {
    const form_data = new FormData();
    
    LIST.entries.filter(entry => entry.is_selected())
        .sort((first, second) => first.position() - second.position())
        .forEach(entry => form_data.append("path", entry.path()));
    
    if (! form_data.has("path")) {
        return;
    }
    
    fetch(PLAY_URL, { method: "POST", body: form_data })
        .then(response => {
            
            if (response.status != 200) {
                response.text().then(error => window.alert(error));
            }
            
        })
        .catch(error => window.alert(error));
}

function mark() {
    const form_data = new FormData();
    
    LIST.entries.filter(entry => entry.is_selected())
        .forEach(entry => form_data.append("path", entry.path()));
    
    if (! form_data.has("path")) {
        return;
    }
    
    fetch(MARK_URL, { method: "POST", body: form_data })
        .then(response => {
            
            if (response.status == 200) {
                location.reload();
            } else {
                response.text().then(error => window.alert(error));
            }
            
        })
        .catch(error => window.alert(error));
}

function move() {
    const folder = prompt("Folder name");
    
    if (! folder) {
        return;
    }
    
    const form_data = new FormData();
    
    form_data.append("folder", folder);
    
    LIST.entries.filter(entry => entry.is_selected())
        .forEach(entry => form_data.append("path", entry.path()));
    
    if (! form_data.has("path")) {
        return;
    }
    
    fetch(MOVE_URL, { method: "POST", body: form_data })
        .then(response => {
            
            if (response.status == 200) {
                location.reload();
            } else {
                response.text().then(error => window.alert(error));
            }
            
        })
        .catch(error => window.alert(error));
}

function lookup() {
    const entry = LIST.entries.find(entry => entry.position() == 1);
    
    if (! entry) {
        return;
    }
    
    const form_data = new FormData();
    
    form_data.append("path", entry.path());
    
    fetch(LOOKUP_URL, { method: "POST", body: form_data })
        .then(response => {
            
            if (response.status != 200) {
                response.text().then(error => window.alert(error));
            }
            
        })
        .catch(error => window.alert(error));
}

function copy() {
    if (! navigator.clipboard) {
        window.alert("Access to the clipboard is only available in secure contexts or localhost")
        return;
    }
    
    const text = LIST.entries.filter(entry => entry.is_selected())
        .sort((first, second) => first.position() - second.position())
        .map(entry => entry.path())
        .join("\n");
    
    navigator.clipboard.writeText(text);
}
