import styled from "@emotion/styled";
import { IronCalc, IronCalcIcon, init, Model } from "@ironcalc/workbook";
import { useEffect, useState } from "react";

async function get_webdav(fileId: string): Promise<Uint8Array> {
  return new Uint8Array(
    await (await fetch(`/index.php/apps/app_api/proxy/ironcalc/api/webdav/${fileId}`)).arrayBuffer(),
  );
}

function createModelWithSafeTimezone(name: string): Model {
  try {
    const tz = Intl.DateTimeFormat().resolvedOptions().timeZone;
    return new Model(name, "en", tz);
  } catch {
    console.warn("Failed to get timezone, defaulting to UTC");
    return new Model(name, "en", "UTC");
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
    <Container>
      <IronCalc model={model} />
    </Container>
  );
}

const Loading = styled("div")`
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