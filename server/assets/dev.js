(() => {
  const container = document.getElementById("content");
  document.getElementById("content").innerHTML = `<div id="root"></div>`;

  const scriptPreamble = document.createElement("script");
  scriptPreamble.setAttribute("type", "module");
  scriptPreamble.textContent = `
    import RefreshRuntime from 'http://localhost:5173/@react-refresh'
    RefreshRuntime.injectIntoGlobalHook(window)
    window.$RefreshReg$ = () => {}
    window.$RefreshSig$ = () => (type) => type
    window.__vite_plugin_react_preamble_installed__ = true
  `;
  document.head.appendChild(scriptPreamble);

  const scriptVite = document.createElement("script");
  scriptVite.setAttribute("type", "module");
  scriptVite.setAttribute("src", "http://localhost:5173/@vite/client");
  document.head.appendChild(scriptVite);

  const scriptMain = document.createElement("script");
  scriptMain.setAttribute("type", "module");
  scriptMain.setAttribute("src", "http://localhost:5173/src/main.tsx");
  document.head.appendChild(scriptMain);
})();
