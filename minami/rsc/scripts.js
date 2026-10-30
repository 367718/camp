"use strict";

document.addEventListener("DOMContentLoaded", () => {
    
    // -------------------- state and selectors --------------------
    
    const listNode = document.querySelector(".list");
    const filterNode = document.querySelector(".filter");
    const collator = new Intl.Collator("en", { numeric: true, sensitivity: "base" });
    
    let entries = [];
    let filterTimeout;
    let refreshAbortController = null;
    let lastSelectedIndex = -1;
    
    // -------------------- colors --------------------
    
    const colors = ["rin", "nadeshiko", "aoi", "chiaki", "ena"];
    document.body.classList.add(colors[Math.floor(Math.random() * colors.length)]);
    
    // -------------------- core functions --------------------
    
    function getEntryText(entry, clean) {
        
        let text = entry.textContent;
        
        if (clean) {
            return text
                // container
                .replace(/^.+\\/, "")
                // extension
                .replace(/\.[^.]+$/, "")
                // brackets and parens
                .replace(/\[[^\]]*\]\s*|\([^\)]*\)\s*/g, "")
                // episode number
                .replace(/\s*-*\s*\d+\s*$/, "")
                // stray whitespace
                .trim();
        }
        
        return text;
        
    }
    
    function sanitize(text) {
        
        return text
            // decompose combined characters (é => e + ´)
            .normalize("NFD")
            // remove diacritical marks (https://www.unicode.org/charts/PDF/U0300.pdf)
            .replace(/[\u0300-\u036f]/g, "")
            .toLowerCase();
        
    }
    
    function applyFilter() {
        
        const criteria = sanitize(filterNode.value);
        
        entries.forEach(entry => {
            
            const match = sanitize(getEntryText(entry, false)).includes(criteria);
            entry.toggleAttribute("data-filtered", ! match);
            
            // deselect if filtered out
            if (! match) entry.removeAttribute("data-selected");
            
        });
        
    }
    
    async function refreshList() {
        
        // try to prevent further calls if one is already in flight
        refreshAbortController?.abort();
        refreshAbortController = new AbortController();
        
        try {
            
            const response = await fetch(listNode.dataset.refresh, {
                signal: refreshAbortController.signal
            });
            
            if (! response.ok) throw new Error(await response.text());
            
            // parse
            
            const parsed = new DOMParser().parseFromString(await response.text(), "text/html");
            let children = Array.from(parsed.body.children);
            
            // sort
            
            if (listNode.dataset.sorted === "true") {
                children.sort((a, b) => a.children.length - b.children.length || collator.compare(a.textContent, b.textContent));
            }
            
            // update
            
            listNode.replaceChildren(...children);
            entries = children;
            lastSelectedIndex = -1;
            
            // filter
            
            applyFilter();
            
        } catch (error) {
            
            if (error.name === "AbortError") return;
            
            listNode.replaceChildren();
            entries = [];
            lastSelectedIndex = -1;
            
            alert(error.message);
            
        }
        
    }
    
    // -------------------- event listeners --------------------
    
    // current section
    
    document.querySelector(".sections a:not([href])")
        .addEventListener("click", () => refreshList());
    
    // filter
    
    filterNode.addEventListener("input", () => {
        clearTimeout(filterTimeout);
        filterTimeout = setTimeout(applyFilter, 500);
    });
    
    // actions
    
    for (const button of document.querySelectorAll(".actions > a")) {
        button.addEventListener("click", async () => {
            
            const selected = entries.filter(entry => entry.hasAttribute("data-selected"));
            
            // no entry was selected, bail
            if (selected.length === 0) return;
            
            // ask for confirmation
            if (button.dataset.confirm === "true" && ! confirm("Are you sure you want to proceed?")) return;
            
            const formData = new FormData();
            
            // ask for key
            if (button.dataset.key === "true") {
                const key = prompt("The requested action requires a key");
                if (key === null) return;
                formData.append("key", key);
            }
            
            // ask for value
            if (button.dataset.value === "true") {
                const value = prompt("The requested action requires a value");
                if (value === null) return;
                formData.append("value", value);
            }
            
            // add every selected entry
            selected.forEach(entry => formData.append("matcher", getEntryText(entry, false)));
            
            try {
                
                const response = await fetch(button.dataset.url, { method: "POST", body: formData });
                if (! response.ok) throw new Error(await response.text());
                if (button.dataset.refresh === "true") refreshList();
                
            } catch (error) {
                
                alert(error.message);
                
            }
            
        });
    }
    
    // toggles
    
    for (const button of document.querySelectorAll(".toggles > a")) {
        button.addEventListener("click", () => {
            
            const state = button.dataset.enabled === "true" ? "false" : "true";
            
            button.dataset.enabled = state;
            listNode.setAttribute(button.dataset.attr, state);
            
            // deselect hidden items
            entries.filter(entry => entry.hasAttribute("data-selected") && ! entry.checkVisibility())
                .forEach(entry => entry.removeAttribute("data-selected"));
            
        });
    }
    
    // entry selection
    
    listNode.addEventListener("click", (event) => {
        
        const targetEntry = event.target.closest(".list > a");
        if (! targetEntry || ! entries.includes(targetEntry) || ! targetEntry.checkVisibility()) return;
        
        const targetIndex = entries.indexOf(targetEntry);
        
        if (! event.ctrlKey && ! event.shiftKey) {
            
            // select target and deselect every other entry
            
            entries.forEach(entry => entry.removeAttribute("data-selected"));
            targetEntry.setAttribute("data-selected", "");
            lastSelectedIndex = targetIndex;
            
        } else if (event.ctrlKey && ! event.shiftKey) {
            
            // select target if deselected
            // deselect target if selected
            
            targetEntry.toggleAttribute("data-selected");
            lastSelectedIndex = targetIndex;
            
        } else if (event.shiftKey) {
            
            // mimic windows explorer behavior
            // shift (without ctrl): replace selection with the range between the anchor and the current target
            // shift + ctrl: keep existing selection, adding the range between the anchor and the current target
            // if target is above anchor, start and end are swapped
            
            if (lastSelectedIndex === -1) {
                lastSelectedIndex = targetIndex;
            }
            
            let start = lastSelectedIndex;
            let end = targetIndex;
            
            if (start > end) [start, end] = [end, start];
            
            if (! event.ctrlKey) {
                entries.forEach((entry) => entry.removeAttribute("data-selected"));
            }
            
            // "slice" does not include the element positioned at "end" index, hence the "+ 1"
            entries.slice(start, end + 1).forEach((entry) => {
                if (entry.checkVisibility()) entry.setAttribute("data-selected", "");
            });
            
        }
        
    });
    
    // copy
    
    document.body.addEventListener("keydown", async (event) => {
        
        // bail if filter input is involved
        if (event.target === filterNode) return;
        
        if (event.ctrlKey && (event.code === "KeyC" || event.code === "KeyX")) {
            
            event.preventDefault();
            
            if (! navigator.clipboard) return alert("Clipboard unavailable in this context.");
            
            const text = entries
                .filter(entry => entry.hasAttribute("data-selected"))
                .map(entry => getEntryText(entry, event.code === "KeyX"))
                .join("\n");
            
            try {
                
                await navigator.clipboard.writeText(text);
                
            } catch (error) {
                
                alert(error.message);
                
            }
            
        }
        
    });
    
    // -------------------- initialization --------------------
    
    // load current section list items
    refreshList();
    
});
