"use strict";

document.addEventListener("DOMContentLoaded", () => {
    
    // -------------------- state and selectors --------------------
    
    const listNode = document.querySelector(".list");
    const filterNode = document.querySelector(".filter");
    const collator = new Intl.Collator("en", { numeric: true, sensitivity: "base" });
    let entries = [];
    let filterTimeout;
    
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
                .replace(/\s*-*\s*\d+\s*$/, "");
        }
        
        return text;
        
    }
    
    function applyFilter() {
        
        const sanitize = text => text
            // decompose combined characters (é => e + ´)
            .normalize("NFD")
            // remove diacritical marks (https://www.unicode.org/charts/PDF/U0300.pdf)
            .replace(/[\u0300-\u036f]/g, "")
            .toLowerCase();
        
        const criteria = sanitize(filterNode.value);
        
        entries.forEach(entry => {
            
            const match = sanitize(getEntryText(entry, false)).includes(criteria);
            entry.toggleAttribute("data-filtered", ! match);
            
            // deselect if filtered out
            if (! match) entry.removeAttribute("data-selected");
            
        });
        
    }
    
    async function refreshList() {
        
        try {
            
            const response = await fetch(listNode.dataset.refresh);
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
            
            // filter
            
            applyFilter();
            
        } catch (error) {
            
            listNode.replaceChildren();
            entries = [];
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
            
            if (button.dataset.confirm === "true" && ! confirm("Are you sure you want to proceed?")) return;
            
            const formData = new FormData();
            
            if (button.dataset.key === "true") {
                const key = prompt("The requested action requires a key");
                if (key === null) return;
                formData.append("key", key);
            }
            
            if (button.dataset.value === "true") {
                const value = prompt("The requested action requires a value");
                if (value === null) return;
                formData.append("value", value);
            }
            
            entries.filter(entry => entry.hasAttribute("data-selected"))
                .forEach(entry => formData.append("matcher", getEntryText(entry, false)));
            
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
            entries.filter(entry => entry.hasAttribute("data-selected") && entry.offsetParent === null)
                .forEach(entry => entry.removeAttribute("data-selected"));
            
        });
    }
    
    // entry selection
    
    listNode.addEventListener("click", (event) => {
        
        const targetEntry = entries.find(entry => entry === event.target || entry.contains(event.target));
        
        if (targetEntry) {
            
            if (! event.ctrlKey && ! event.shiftKey) {
                
                // select target and deselect every other entry
                
                entries.forEach(entry => entry.removeAttribute("data-selected"));
                targetEntry.setAttribute("data-selected", "");
                
            } else if (event.ctrlKey) {
                
                // select target if deselected
                // deselect target if selected
                
                targetEntry.toggleAttribute("data-selected");
                
            } else if (event.shiftKey) {
                
                // if target is positioned after the first selected entry, make a new selection from the first selected entry up to target
                // if target is positioned before the first selected entry, make a new selection from the target up to last selected entry
                // if no entry is selected, make a new selection from the first visible entry up to target
                
                let start = entries.findIndex(entry => entry.hasAttribute("data-selected"));
                let end = entries.indexOf(targetEntry);
                
                if (start === -1) start = entries.findIndex(entry => entry.offsetParent !== null);
                if (start > end) [start, end] = [end, entries.findLastIndex(entry => entry.hasAttribute("data-selected"))];
                
                entries.forEach(entry => entry.removeAttribute("data-selected"));
                entries.slice(start, end + 1).forEach(entry => {
                    if (entry.offsetParent !== null) entry.setAttribute("data-selected", "");
                });
                
            }
            
        }
        
    });
    
    // copy
    
    document.body.addEventListener("keydown", (event) => {
        
        // bail if filter input is involved
        if (event.target === filterNode) return;
        
        if (event.ctrlKey && (event.code === "KeyC" || event.code === "KeyX")) {
            
            event.preventDefault();
            
            if (! navigator.clipboard) return alert("Clipboard unavailable in this context.");
            
            const text = entries
                .filter(entry => entry.hasAttribute("data-selected"))
                .map(entry => getEntryText(entry, event.code === "KeyX"))
                .join("\n");
            
            navigator.clipboard.writeText(text);
            
        }
        
    });
    
    // -------------------- initialization --------------------
    
    refreshList();
    
});
