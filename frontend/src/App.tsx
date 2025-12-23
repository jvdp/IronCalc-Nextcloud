import "./App.css";
import styled from "@emotion/styled";
import { IronCalc, IronCalcIcon, init, Model } from "@ironcalc/workbook";
import { useEffect, useState } from "react";
import { get_webdav, } from "./components/rpc";
import { createModelWithSafeTimezone } from "./components/storage";

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
          const model_bytes = await get_webdav(fileId);
          const importedModel = Model.from_bytes(model_bytes);
          localStorage.removeItem("selected");
          setModel(importedModel);
        } catch (_e) {
          console.log(_e)
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

  // We could use context for model, but the problem is that it should initialized to null.
  // Passing the property down makes sure it is always defined.
  return (
    <IronCalc model={model} />
  );
}

const Loading = styled("div")`
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  font-family: "Inter";
  font-size: 14px;
`;

export default App;
