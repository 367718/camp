// -------------------- remote resources --------------------


const ADD_URL = "/watchlist/add";
const EDIT_URL = "/watchlist/edit";
const REMOVE_URL = "/watchlist/remove";
const LOOKUP_URL = "/watchlist/lookup";


// -------------------- hotkeys --------------------


const SELECT_ACTIVATE_HOTKEY = (event) => ["KeyD", "ArrowRight"].includes(event.code);
const SELECT_DEACTIVATE_HOTKEY = (event) => ["KeyA", "ArrowLeft"].includes(event.code);
const SELECT_CLEAR_HOTKEY = (event) => event.code === "KeyF" && ! event.ctrlKey;

const FOCUS_UP_HOTKEY = (event) => ["KeyW", "ArrowUp"].includes(event.code);
const FOCUS_DOWN_HOTKEY = (event) => ["KeyS", "ArrowDown"].includes(event.code);
const FOCUS_JUMP_UP_HOTKEY = (event) => event.code === "PageUp";
const FOCUS_JUMP_DOWN_HOTKEY = (event) => event.code === "PageDown";

const ADD_HOTKEY = (event) => event.code === "Insert";
const EDIT_HOTKEY = (event) => event.code === "F2";
const REMOVE_HOTKEY = (event) => event.code === "Delete";
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
    
    select = () => this.node.dataset.selected = "";
    deselect = () => this.node.removeAttribute("data-selected");
    
    is_selected = () => this.node.hasAttribute("data-selected");
    is_visible = () => this.node.offsetParent != null;
    
    title = () => this.node.firstElementChild.firstElementChild.textContent;
    progress = () => this.node.firstElementChild.lastElementChild.textContent;
    
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
    
    LIST.entries.forEach(entry => entry.onclick(() => select(entry, SelectAction.Activate)));
    
    LIST.onkeydown((event) => {
        
        // ---------- select entry ----------
        
        // activate
        
        if (SELECT_ACTIVATE_HOTKEY(event)) {
            select(LIST.entry(event.target), SelectAction.Activate);
            return event.preventDefault();
        }
        
        // deactivate
        
        if (SELECT_DEACTIVATE_HOTKEY(event)) {
            select(LIST.entry(event.target), SelectAction.Deactivate);
            return event.preventDefault();
        }
        
        // clear
        
        if (SELECT_CLEAR_HOTKEY(event)) {
            select(null, SelectAction.Clear);
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
        
        // ---------- add series ----------
        
        if (ADD_HOTKEY(event)) {
            add();
            return event.preventDefault();
        }
        
        // ---------- edit series ----------
        
        if (EDIT_HOTKEY(event)) {
            edit();
            return event.preventDefault();
        }
        
        // ---------- remove series ----------
        
        if (REMOVE_HOTKEY(event)) {
            remove();
            return event.preventDefault();
        }
        
        // ---------- copy title to clipboard ----------
        
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
            if (regex.exec(entry.title())) {
                entry.unfilter();
            } else {
                entry.filter();
                entry.deselect();
            }
        }
        
    }, 250);
}

function toggle(criteria) {
    LIST.toggle(criteria);
    
    if (entry = LIST.entries.find(entry => entry.is_selected() && ! entry.is_visible())) {
        entry.deselect();
    }
}

function select(target, action) {
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
    
    if (target.is_selected()) {
        
        target.deselect();
        
    } else {
        
        // disallow multiple selection by deselecting everything first
        LIST.entries.filter(entry => entry.is_selected())
            .forEach(entry => entry.deselect());
        
        target.select();
        
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

function add() {
    const title = prompt("New series title");
    
    if (! title) {
        return;
    }
    
    const form_data = new FormData();
    
    form_data.append("title", title);
    
    fetch(ADD_URL, { method: "POST", body: form_data })
        .then(response => {
            
            if (response.status == 200) {
                location.reload();
            } else {
                response.text().then(error => window.alert(error));
            }
            
        })
        .catch(error => window.alert(error));
}

function edit() {
    const entry = LIST.entries.find(entry => entry.is_selected());
    
    if (! entry) {
        return;
    }
    
    const progress = prompt("Change series progress", entry.progress());
    
    if (! progress) {
        return;
    }
    
    const form_data = new FormData();
    
    form_data.append("title", entry.title());
    form_data.append("progress", progress);
    
    fetch(EDIT_URL, { method: "POST", body: form_data })
        .then(response => {
            
            if (response.status == 200) {
                location.reload();
            } else {
                response.text().then(error => window.alert(error));
            }
            
        })
        .catch(error => window.alert(error));
}

function remove() {
    const entry = LIST.entries.find(entry => entry.is_selected());
    
    if (! entry || ! confirm("Are you sure you want to remove the selected series?")) {
        return;
    }
    
    const form_data = new FormData();
    
    form_data.append("title", entry.title());
    
    fetch(REMOVE_URL, { method: "POST", body: form_data })
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
    const entry = LIST.entries.find(entry => entry.is_selected());
    
    if (! entry) {
        return;
    }
    
    const form_data = new FormData();
    
    form_data.append("title", entry.title());
    
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
    
    if (entry = LIST.entries.find(entry => entry.is_selected())) {
        navigator.clipboard.writeText(entry.title());
    }
}
