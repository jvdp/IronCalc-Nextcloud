import createCache from "@emotion/cache";
import { CacheProvider } from "@emotion/react";
import { createTheme, ThemeProvider } from "@mui/material/styles";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App.tsx";

// biome-ignore lint: we know the 'root' element exists.
const root = document.getElementById("root")!;
const shadowContainer = root.attachShadow({ mode: "open" });
const shadowRoot = document.createElement("div");
shadowContainer.appendChild(shadowRoot);

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
