import styled from "@emotion/styled";
import { IronCalc, IronCalcIcon, init, Model } from "@ironcalc/workbook";
import { useEffect, useState } from "react";
import { model_load } from "./calls";
import TitleBar from "./TitleBar";

function getDefaultUILocale(): string {
  const lang = navigator.language || navigator.languages[0] || "en-US";
  if (lang.startsWith("es")) {
    return "es-ES";
  } else if (lang.startsWith("fr")) {
    return "fr-FR";
  } else if (lang.startsWith("de")) {
    return "de-DE";
  } else if (lang === "en-GB") {
    return "en-GB";
  } else if (lang.startsWith("it")) {
    return "it-IT";
  }

  return "en-US";
}

// Converts long language codes to short ones used by the Model
export function getShortLocaleCode(longCode: string): string {
  switch (longCode) {
    case "es-ES": {
      return "es";
    }
    case "fr-FR": {
      return "fr";
    }
    case "de-DE": {
      return "de";
    }
    case "it-IT": {
      return "it";
    }
    case "en-GB": {
      return "en-GB";
    }
    default: {
      return "en";
    }
  }
}

function getLanguageFromLocale(locale: string): string {
  return locale.split("-")[0];
}

function createModelWithSafeTimezone(name: string): Model {
  const locale = getDefaultUILocale();
  const language = getLanguageFromLocale(locale);
  const localeShort = getShortLocaleCode(locale);
  try {
    const tz = Intl.DateTimeFormat().resolvedOptions().timeZone;
    return new Model(name, localeShort, tz, language);
  } catch (e) {
    console.warn("Failed to get timezone, defaulting to UTC", e);
    return new Model(name, localeShort, "UTC", language);
  }
}

function App() {
  const [model, setModel] = useState<Model | null>(null);

  useEffect(() => {
    async function start() {
      await init();
      const queryString = window.location.search;
      const urlParams = new URLSearchParams(queryString);
      const fileId = urlParams.get("fileIds")?.toString().split(",")[0];
      // If there is a file id ?fileIds=num we try to load it
      // if there is not, or the loading failed we load an empty model
      if (fileId) {
        // Get a remote model
        try {
          const locale = getDefaultUILocale();
          const language = getLanguageFromLocale(locale);
          const tz =
            Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC";
          const model_bytes = await model_load(fileId, language, tz);
          const importedModel = Model.from_bytes(model_bytes, language);
          localStorage.removeItem("selected");
          setModel(importedModel);
        } catch (_e) {
          console.log(_e);
          alert("Model not found, or failed to load");
        }
      } else {
        const createdModel = createModelWithSafeTimezone("template");
        setModel(createdModel);
      }
    }
    start();
  }, []);

  useEffect(() => {
    if (model) {
      const workbookName = model.getName();
      document.title = workbookName ? `${workbookName} - IronCalc` : "IronCalc";
    } else {
      document.title = "IronCalc";
    }
  }, [model]);

  if (!model) {
    return (
      <Loading>
        <IronCalcIcon style={{ width: 24, height: 24, marginBottom: 16 }} />
        <div>Loading IronCalc</div>
      </Loading>
    );
  }

  return (
    <div
      css={{
        display: "flex",
        flexDirection: "column",
        top: 0,
        bottom: 0,
        left: 0,
        right: 0,
        position: "absolute",
      }}
    >
      <TitleBar />
      <div css={{ flex: "1", background: "blue", position: "relative" }}>
        <Container>
          <IronCalc model={model} />
        </Container>
      </div>
    </div>
  );
}

const Loading = styled.div`
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  font-size: 14px;
`;

const Container = styled.div`
  position: absolute;
  inset: 0px;
  margin: 0px;
  border: none;
  background-color: white;
`;

export default App;
