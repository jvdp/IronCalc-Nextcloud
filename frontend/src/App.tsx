import styled from "@emotion/styled";
import { IronCalc, IronCalcIcon, init, Model } from "@ironcalc/workbook";
import { useEffect } from "react";
import {
  PROXY_BASE,
  workbook_load,
  workbook_rename,
  workbook_save,
} from "./calls";
import { useAsync, useLocationState } from "./hooks";
import TitleBar from "./TitleBar";
import {
  getDefaultUILocale,
  getLanguageFromLocale,
  getShortLocaleCode,
  getTimezone,
} from "./utils";

const wasmUrl = import.meta.env.DEV
  ? undefined
  : `${PROXY_BASE}/assets/wasm_bg.wasm`;

function App() {
  const locale = getDefaultUILocale();
  const language = getLanguageFromLocale(locale);
  const localeShort = getShortLocaleCode(locale);
  const timezone = getTimezone();
  const { location, updateLocation } = useLocationState();
  const {
    run: runLoadModel,
    data: model,
    error,
  } = useAsync(async (): Promise<Model> => {
    const wasmInit = init(wasmUrl);
    if (location) {
      const [, workbookBytes] = await Promise.all([
        wasmInit,
        workbook_load(location.fileId, language, timezone, location.filePath),
      ]);
      return Model.from_bytes(workbookBytes, language);
    }
    await wasmInit;
    return new Model("untitled", localeShort, timezone, language);
  }, []);

  useEffect(() => {
    runLoadModel();
  }, [runLoadModel]);

  useEffect(() => {
    const workbookName = model?.getName();
    if (workbookName) {
      document.title = `${workbookName} - IronCalc`;
    } else {
      document.title = "IronCalc";
    }
  }, [model]);

  if (!model) {
    return (
      <Loading>
        <IronCalcIcon style={{ width: 24, height: 24, marginBottom: 16 }} />
        {error ? (
          <div css={{ color: "#c44" }}>Failed to load: {error}</div>
        ) : (
          <div>Loading IronCalc</div>
        )}
      </Loading>
    );
  }

  const handleCreate = async (newFileName: string) => {
    const newPath = location?.dir
      ? `${location.dir}/${newFileName}`
      : newFileName;
    const newFileId = await workbook_save(
      "0",
      model.toBytes(),
      language,
      newPath,
    );
    updateLocation(newPath, newFileId);
  };

  const handleSave = async () => {
    if (!location) return;
    await workbook_save(
      location.fileId,
      model.toBytes(),
      language,
      location.filePath,
    );
  };

  const handleRename = async (newName: string) => {
    if (!location) return;
    const newPath = location.dir ? `${location.dir}/${newName}` : newName;
    await workbook_rename(location.fileId, newName);
    updateLocation(newPath, location.fileId);
  };

  return (
    <AppContainer>
      <TitleBar
        onSave={handleSave}
        onCreate={handleCreate}
        onRename={handleRename}
        location={location}
      />
      <WorkbookContainer>
        <IronCalc model={model} />
      </WorkbookContainer>
    </AppContainer>
  );
}

const AppContainer = styled.div`
  display: flex;
  flex-direction: column;
  position: absolute;
  inset: 0;
`;

const WorkbookContainer = styled.div`
  flex: 1;
  position: relative;
  & > * {
    position: absolute;
    inset: 0;
    margin: 0;
    border: none;
    background-color: white;
  }
`;

const Loading = styled.div`
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  font-size: 14px;
`;

export default App;
