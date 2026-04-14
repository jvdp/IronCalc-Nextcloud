import createCache from "@emotion/cache";
import { CacheProvider } from "@emotion/react";
import workbookCss from "@ironcalc/workbook/style.css?inline";
import { createTheme, ThemeProvider } from "@mui/material/styles";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App.tsx";

// biome-ignore lint: we know the 'content' element exists in Nextcloud.
const content = document.getElementById("content")!;
const shadowContainer = content.attachShadow({ mode: "open" });
const shadowRoot = document.createElement("div");
shadowContainer.appendChild(shadowRoot);

const sheet = new CSSStyleSheet();
sheet.replaceSync(workbookCss);
shadowContainer.adoptedStyleSheets = [sheet];

const emotionCache = createCache({
  key: "ironcalc",
  container: shadowContainer,
  prepend: true,
});

const theme = createTheme({
  components: {
    MuiPopover: {
      defaultProps: {
        container: shadowRoot,
      },
    },
    MuiPopper: {
      defaultProps: {
        container: shadowRoot,
      },
    },
    MuiModal: {
      defaultProps: {
        container: shadowRoot,
      },
    },
  },
});

createRoot(shadowRoot).render(
  <CacheProvider value={emotionCache}>
    <ThemeProvider theme={theme}>
      <StrictMode>
        <App />
      </StrictMode>
    </ThemeProvider>
  </CacheProvider>,
);
