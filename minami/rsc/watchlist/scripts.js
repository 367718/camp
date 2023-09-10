// -------------------- attributes and nodes --------------------


const LIST = () => document.querySelector(".list");
const TOGGLES = () => Array.from(document.querySelectorAll(".panel input[type='checkbox']"));


// -------------------- event binding --------------------


document.addEventListener("DOMContentLoaded", () => {
    
    document.querySelector(".filter").addEventListener("input", () => filter(), false);
    
    TOGGLES().forEach(entry => entry.addEventListener("click", () => toggle_entries(entry.value), false));
    
    Object.defineProperty(window, "ENTRIES", {
        value: Array.from(LIST().children),
        configurable: false,
        writable: false
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
        
        for (entry of ENTRIES) {
            if (entry.textContent.match(regex)) {
                entry.classList.remove("filtered");
            } else {
                entry.classList.add("filtered");
            }
        }
        
    }, 250);
}

function toggle_entries(criteria) {
    LIST().classList.toggle(criteria);
}
