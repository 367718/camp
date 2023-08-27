// -------------------- remote resources --------------------


const PLAY_URL = "/files/play";
const MARK_URL = "/files/mark";


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

const SelectAction = {
	Toggle: Symbol(0),
	Activate: Symbol(0),
    Deactivate: Symbol(0),
    Clear: Symbol(0),
}

const ExpandAction = {
	Open: Symbol(0),
	Close: Symbol(0),
}

const ExpandTarget = {
	Current: Symbol(0),
	All: Symbol(0),
}


// -------------------- attributes and nodes --------------------


const LIST = () => document.querySelector(".list");
const FILTERS = () => Array.from(document.querySelector(".filters").querySelectorAll("input"));

const POSITION_ATTRIBUTE = "data-position";

// position

function entryGetPosition(entry) {
    return parseInt(entry.getAttribute(POSITION_ATTRIBUTE)) || 0;
}

function entrySetPosition(entry, position) {
    entry.setAttribute(POSITION_ATTRIBUTE, parseInt(position) || 0)
}

// path

function entryGetPath(entry) {
    return entry.textContent;
}

// selection

function entryIsSelected(entry) {
    return entryGetPosition(entry) > 0;
}

function entryDeselect(entry) {
    entry.removeAttribute(POSITION_ATTRIBUTE);
}

// visibility

function entryIsVisible(entry) {
    return entry.offsetParent != null;
}


// -------------------- event binding --------------------


document.addEventListener("DOMContentLoaded", () => {
    
    FILTERS().forEach(entry => entry.addEventListener("click", () => filter(entry.value), false));
    
    Object.defineProperty(window, "ENTRIES", {
        value: Array.from(LIST().children),
        configurable: false,
        writable: false
    });
    
    ENTRIES.forEach(entry => entry.addEventListener("click", () => select(entry, SelectAction.Toggle), false));
    
});

document.addEventListener("keydown", (event) => {
    
    // ---------- select entry ----------
    
    // activate
        
    if (SELECT_ACTIVATE_HOTKEY(event)) {
        select(event.target, SelectAction.Activate);
        return event.preventDefault();
    }
    
    // deactivate
    
    if (SELECT_DEACTIVATE_HOTKEY(event)) {
        select(event.target, SelectAction.Deactivate);
        return event.preventDefault();
    }
    
    // clear
    
    if (SELECT_CLEAR_HOTKEY(event)) {
        select(null, SelectAction.Clear);
        return event.preventDefault();
    }
    
    // ---------- focus ----------
    
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


// -------------------- functionality --------------------


function filter(criteria) {
    LIST().classList.toggle(criteria);
    
    ENTRIES.filter(entry => entryIsSelected(entry) && ! entryIsVisible(entry))
        .forEach(entry => entry.click());
}

function select(target, action) {
    const selected = ENTRIES.filter(entry => entryIsSelected(entry));
    
    // deselect every entry
    
    if (action == SelectAction.Clear) {
        selected.forEach(entry => entryDeselect(entry));
        return;
    }
    
    // bail if state needs no change
    
    if ((action == SelectAction.Activate && entryIsSelected(target)) || (action == SelectAction.Deactivate && ! entryIsSelected(target))) {
        return;
    }
    
    // if entry was unselected, re-calculate every higher position
    // if entry was selected, calculate next position
    
    if (entryIsSelected(target)) {
        
        const changed = entryGetPosition(target);
        
        for (entry of selected) {
            const current = entryGetPosition(entry);
            if (current > changed) {
                entrySetPosition(entry, current - 1);
            }
        }
        
        entryDeselect(target);
        
    } else {
        
        let position = 0;
        
        for (entry of selected) {
            position = Math.max(position, entryGetPosition(entry));
        }
        
        entrySetPosition(target, position + 1);
        
    }
    
    target.focus();
}

function focus(distance, direction) {
    const current = ENTRIES.find(entry => entry == document.activeElement);
    
    if (current) {
        
        const visible = ENTRIES.filter(entry => entryIsVisible(entry));
        
        const change = visible.indexOf(current) + distance * direction;
        const index = Math.min(Math.max(change, 0), visible.length - 1);
        
        visible[index].focus();
        
    } else {
        
        const first = ENTRIES.find(entry => entryIsVisible(entry));
        
        if (first) {
            first.focus();
        }
        
    }
}

function play() {
    const form_data = new FormData();
    
    ENTRIES.filter(entry => entryIsSelected(entry))
        .sort((first, second) => entryGetPosition(first) - entryGetPosition(second))
        .forEach(entry => form_data.append("path", entryGetPath(entry)));
    
    if (form_data.has("path")) {
        fetch(PLAY_URL, { method: "POST", body: form_data })
            .then(response => {
                
                if (response.status != 200) {
                    response.text().then(error => window.alert(error));
                }
                
            })
            .catch(error => window.alert(error));
    }
}

function mark() {
    const form_data = new FormData();
    
    ENTRIES.filter(entry => entryIsSelected(entry))
        .forEach(entry => form_data.append("path", entryGetPath(entry)));
    
    if (form_data.has("path")) {
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
}

function copy() {
    if (! navigator.clipboard) {
        window.alert("Access to the clipboard is only available in secure contexts or localhost")
        return;
    }
    
    const text = ENTRIES.filter(entry => entryIsSelected(entry))
        .sort((first, second) => entryGetPosition(first) - entryGetPosition(second))
        .map(entry => entryGetPath(entry))
        .join("\n");
    
    navigator.clipboard.writeText(text);
}
