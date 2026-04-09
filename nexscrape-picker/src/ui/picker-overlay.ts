import { OverlayMessage, ElementInfo } from "../contracts.js";

// Global WebSocket instance
let ws: WebSocket | null = null;
let isPickingEnabled = false;

// UI Elements
const highlightBox = document.createElement("div");
const tooltip = document.createElement("div");

function initOverlayUI() {
  highlightBox.id = "nex-highlight-box";
  tooltip.id = "nex-tooltip";
  
  document.body.appendChild(highlightBox);
  document.body.appendChild(tooltip);

  // Setup styles dynamically if CSS file wasn't loaded (fallback)
  Object.assign(highlightBox.style, {
    position: "absolute",
    pointerEvents: "none",
    border: "2px solid #eb5e28", // NexScrape Brand Color
    backgroundColor: "rgba(235, 94, 40, 0.15)",
    zIndex: "999999",
    transition: "all 0.1s ease-out",
    display: "none"
  });

  Object.assign(tooltip.style, {
    position: "absolute",
    pointerEvents: "none",
    backgroundColor: "#252422",
    color: "#fffcf2",
    padding: "4px 8px",
    borderRadius: "4px",
    fontSize: "12px",
    fontFamily: "monospace",
    zIndex: "1000000",
    display: "none",
    boxShadow: "0 2px 10px rgba(0,0,0,0.2)"
  });
}

function connectWebSocket() {
  const port = (window as any).__NEX_WS_PORT;
  if (!port) {
    console.error("NexPicker: WebSocket port not found.");
    return;
  }

  ws = new WebSocket(`ws://127.0.0.1:${port}`);

  ws.onopen = () => {
    console.log("NexPicker: Connected to backend.");
    isPickingEnabled = true; // Auto-enable picking for now
    updateCursor();
  };

  ws.onmessage = (event) => {
    try {
      const msg = JSON.parse(event.data);
      handleBackendMessage(msg);
    } catch (e) {
      console.error("Error parsing backend message:", e);
    }
  };

  ws.onclose = () => {
    console.log("NexPicker: Disconnected.");
    isPickingEnabled = false;
    updateCursor();
    hideHighlight();
  };
}

function handleBackendMessage(msg: any) {
  if (msg.type === "picker:enable") {
    isPickingEnabled = true;
    updateCursor();
  } else if (msg.type === "picker:disable") {
    isPickingEnabled = false;
    updateCursor();
    hideHighlight();
  }
}

function updateCursor() {
  if (isPickingEnabled) {
    document.body.style.cursor = "crosshair";
  } else {
    document.body.style.cursor = "";
  }
}

function hideHighlight() {
  highlightBox.style.display = "none";
  tooltip.style.display = "none";
}

function extractElementInfo(el: HTMLElement): ElementInfo {
  const rect = el.getBoundingClientRect();
  const computed = window.getComputedStyle(el);
  
  // Extract attributes
  const attributes: Record<string, string> = {};
  for (let i = 0; i < el.attributes.length; i++) {
    const attr = el.attributes[i];
    attributes[attr.name] = attr.value;
  }

  // Get DomPath
  const domPath = [];
  let curr: HTMLElement | null = el;
  while (curr && curr !== document.documentElement) {
    let index = 0;
    let sibling = curr.previousElementSibling;
    while (sibling) {
      if (sibling.tagName === curr.tagName) index++;
      sibling = sibling.previousElementSibling;
    }
    domPath.unshift({
      tag: curr.tagName.toLowerCase(),
      id: curr.id || null,
      classes: Array.from(curr.classList),
      index,
      siblingCount: curr.parentElement ? curr.parentElement.children.length : 0
    });
    curr = curr.parentElement;
  }

  let textContent = (el.textContent || "").trim();
  if (textContent.length > 500) textContent = textContent.slice(0, 500) + "...";

  return {
    tag: el.tagName.toLowerCase(),
    id: el.id || null,
    classes: Array.from(el.classList),
    attributes,
    textContent,
    innerHTML: el.innerHTML.slice(0, 2000),
    outerHTML: el.outerHTML.slice(0, 3000),
    role: el.getAttribute("role"),
    ariaLabel: el.getAttribute("aria-label"),
    rect: {
      x: rect.x + window.scrollX,
      y: rect.y + window.scrollY,
      width: rect.width,
      height: rect.height
    },
    domPath,
    parent: el.parentElement ? {
      tag: el.parentElement.tagName.toLowerCase(),
      id: el.parentElement.id || null,
      classes: Array.from(el.parentElement.classList),
      role: el.parentElement.getAttribute("role"),
      childCount: el.parentElement.children.length
    } : null,
    siblingCount: el.parentElement ? el.parentElement.children.length : 0,
    siblingIndex: domPath[domPath.length - 1]?.index || 0,
    childCount: el.children.length,
    computedStyle: {
      display: computed.display,
      visibility: computed.visibility,
      opacity: computed.opacity,
      fontSize: computed.fontSize,
      fontWeight: computed.fontWeight,
      color: computed.color
    }
  };
}

// Global Event Listeners
document.addEventListener("mouseover", (e) => {
  if (!isPickingEnabled) return;
  const target = e.target as HTMLElement;
  if (!target || target === document.documentElement || target === document.body) {
    hideHighlight();
    return;
  }
  
  // Ignore clicks on our own overlay
  if (target.id === "nex-highlight-box" || target.id === "nex-tooltip") return;

  const rect = target.getBoundingClientRect();
  
  highlightBox.style.display = "block";
  highlightBox.style.top = `${rect.top + window.scrollY}px`;
  highlightBox.style.left = `${rect.left + window.scrollX}px`;
  highlightBox.style.width = `${rect.width}px`;
  highlightBox.style.height = `${rect.height}px`;

  tooltip.style.display = "block";
  tooltip.style.top = `${rect.top + window.scrollY - 25}px`;
  tooltip.style.left = `${rect.left + window.scrollX}px`;
  
  let tooltipText = target.tagName.toLowerCase();
  if (target.id) tooltipText += `#${target.id}`;
  else if (target.classList.length > 0) tooltipText += `.${target.classList[0]}`;
  tooltip.textContent = tooltipText;

  // Send hover info to backend
  if (ws && ws.readyState === WebSocket.OPEN) {
    const msg: OverlayMessage = {
      type: "element:hover",
      data: {
        selector: tooltipText,
        preview: target.textContent?.slice(0, 30).trim() || ""
      }
    };
    ws.send(JSON.stringify(msg));
  }
});

document.addEventListener("mouseout", (e) => {
  if (!isPickingEnabled) return;
  // hideHighlight(); // Usually better to let mouseover handle changes to reduce flicker
});

document.addEventListener("click", (e) => {
    if (!isPickingEnabled) return;
    
    // Prevent default actions like navigating away on link click
    e.preventDefault();
    e.stopPropagation();

    const target = e.target as HTMLElement;
    if (!target) return;

    // Build the info object
    const info = extractElementInfo(target);

    // Send to backend
    if (ws && ws.readyState === WebSocket.OPEN) {
        const msg: OverlayMessage = {
            type: "element:click",
            data: info
        };
        ws.send(JSON.stringify(msg));
    }

    // Temporarily change highlight explicitly to show success flash
    highlightBox.style.backgroundColor = "rgba(42, 157, 143, 0.4)";
    highlightBox.style.borderColor = "#2a9d8f";
    setTimeout(() => {
        highlightBox.style.backgroundColor = "rgba(235, 94, 40, 0.15)";
        highlightBox.style.borderColor = "#eb5e28";
    }, 300);
}, true); // Use capture phase to ensure we intercept

// Initialize
initOverlayUI();
connectWebSocket();
