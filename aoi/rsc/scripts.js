"use strict";


const BUTTONS_NODE_SELECTOR = "button";
const BUTTONS_PATH_ATTRIBUTE = "data-path";

document.addEventListener("DOMContentLoaded", () => {
  
  for (let button of document.querySelectorAll(BUTTONS_NODE_SELECTOR)) {
    button.addEventListener("click", () => {
      
      const path = button.getAttribute(BUTTONS_PATH_ATTRIBUTE);
      
      fetch(path, { method: "POST" })
        .then(response => {
          if (response.status != 200) {
            response.text().then(error => window.alert(error));
          }
        })
        .catch(error => window.alert(error));
      
    });
  }
  
});
