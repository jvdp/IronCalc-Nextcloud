import {
  CopyPlus,
  Download,
  LoaderCircle,
  Save,
  TriangleAlert,
  X,
} from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useAsync } from "./hooks";
import {
  downloadFile,
  ensureExtension,
  type Location,
  navigateToFolder,
} from "./utils";

function IconButton({
  icon: Icon,
  title,
  disabled,
  onClick,
}: {
  icon: React.ComponentType<{ size: number }>;
  title: string;
  disabled?: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      title={title}
      disabled={disabled}
      onClick={onClick}
      css={{
        display: "inline-flex",
        width: 24,
        minWidth: 24,
        height: 24,
        padding: 0,
        alignItems: "center",
        justifyContent: "center",
        border: "none",
        borderRadius: 4,
        backgroundColor: "transparent",
        color: "#333",
        opacity: disabled ? 0.3 : 1,
        cursor: disabled ? "default" : "pointer",
        "&:hover": { backgroundColor: disabled ? "transparent" : "#f0f0f0" },
        "&:active": { backgroundColor: disabled ? "transparent" : "#e0e0e0" },
      }}
    >
      <Icon size={16} />
    </button>
  );
}

const titleInputStyle = {
  border: "none",
  outline: "none",
  background: "transparent",
  fontWeight: "bold",
  fontSize: "inherit",
  fontFamily: "inherit",
  color: "#000",
  padding: "2px 6px",
  borderRadius: 4,
  maxWidth: "100%",
  textAlign: "center" as const,
  cursor: "text",
  caretColor: "#F2994A",
  "&::placeholder": { color: "#999", fontWeight: "normal" },
};

const titleLabelStyle = {
  position: "absolute" as const,
  inset: "0 12px",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  cursor: "text",
};

function TitleInput({
  value,
  onChange,
  onConfirm,
  onCancel,
  disabled,
  placeholder,
  inputRef,
}: {
  value: string;
  onChange: (value: string) => void;
  onConfirm?: (name: string) => void;
  onCancel?: () => void;
  disabled: boolean;
  placeholder?: string;
  inputRef: React.RefObject<HTMLInputElement | null>;
}) {
  const skipBlur = useRef(false);

  const confirm = () => {
    if (!onConfirm) return;
    const trimmed = value.trim();
    if (trimmed) onConfirm(ensureExtension(trimmed));
  };

  return (
    <label css={titleLabelStyle}>
      <input
        ref={inputRef}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={
          onConfirm || onCancel
            ? (e) => {
                if (e.key === "Enter" && onConfirm) {
                  e.preventDefault();
                  skipBlur.current = true;
                  inputRef.current?.blur();
                  confirm();
                }
                if (e.key === "Escape" && onCancel) {
                  skipBlur.current = true;
                  inputRef.current?.blur();
                  onCancel();
                }
              }
            : undefined
        }
        onBlur={
          onConfirm
            ? () => {
                if (skipBlur.current) {
                  skipBlur.current = false;
                  return;
                }
                confirm();
              }
            : undefined
        }
        disabled={disabled}
        placeholder={placeholder}
        spellCheck={false}
        css={{
          ...titleInputStyle,
          width: value ? `${value.length + 2}ch` : "16ch",
        }}
      />
    </label>
  );
}

function StatusIndicator({
  error,
  pending,
  message,
}: {
  error: string | null;
  pending: boolean;
  message?: string;
}) {
  if (error) {
    return (
      <div
        css={{
          display: "flex",
          alignItems: "center",
          gap: 4,
          color: "#c44",
          fontSize: 12,
          whiteSpace: "nowrap",
        }}
      >
        <TriangleAlert size={14} />
        {error}
      </div>
    );
  }
  if (pending) {
    return (
      <LoaderCircle
        size={16}
        css={{
          color: "#999",
          "@keyframes spin": {
            from: { transform: "rotate(0deg)" },
            to: { transform: "rotate(360deg)" },
          },
          animation: "spin 1s linear infinite",
        }}
      />
    );
  }
  if (message) {
    return (
      <span css={{ color: "#999", fontSize: 12, whiteSpace: "nowrap" }}>
        {message}
      </span>
    );
  }
  return null;
}

type InputMode = "renaming" | "duplicating";

function ButtonsContainer({ children }: { children: React.ReactNode }) {
  return (
    <div
      css={{
        marginLeft: "auto",
        display: "flex",
        alignItems: "center",
        gap: 4,
        padding: "0 12px",
        position: "relative",
        zIndex: 1,
        background: "#fff",
      }}
    >
      {children}
    </div>
  );
}

function NewFileTitleBar({
  onCreate,
}: {
  onCreate: (name: string) => Promise<void>;
}) {
  const [editedName, setEditedName] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);

  const { run, pending, error } = useAsync(
    (name: string) => onCreate(name),
    [onCreate],
  );

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  const handleSave = () => run(ensureExtension(editedName.trim()));

  return (
    <>
      <TitleInput
        value={editedName}
        onChange={setEditedName}
        disabled={pending}
        placeholder="Enter a file name"
        inputRef={inputRef}
      />
      <ButtonsContainer>
        <StatusIndicator error={error} pending={pending} message="New file" />
        <IconButton
          icon={Save}
          title="Save"
          disabled={pending || editedName.trim().length == 0}
          onClick={handleSave}
        />
        <IconButton icon={X} title="Close" onClick={navigateToFolder} />
      </ButtonsContainer>
    </>
  );
}

function ExistingFileTitleBar({
  onSave,
  onCreate,
  onRename,
  location,
}: {
  onSave: () => Promise<void>;
  onCreate: (name: string) => Promise<void>;
  onRename: (name: string) => Promise<void>;
  location: Location;
}) {
  const { fileName, filePath } = location;
  const [editedName, setEditedName] = useState(fileName);
  const [mode, setMode] = useState<InputMode>("renaming");
  const inputRef = useRef<HTMLInputElement>(null);

  const { run, pending, error } = useAsync(
    (fn: () => Promise<void>) => fn(),
    [],
  );

  useEffect(() => {
    setEditedName(fileName);
  }, [fileName]);

  const handleSave = () => run(onSave);

  const handleConfirm = async (name: string) => {
    if (name === fileName) {
      setEditedName(fileName);
      setMode("renaming");
      return;
    }
    const fn =
      mode === "duplicating" ? () => onCreate(name) : () => onRename(name);
    const result = await run(fn);
    if (!result.ok) setEditedName(fileName);
    setMode("renaming");
  };

  const handleCancel = () => {
    setMode("renaming");
    setEditedName(fileName);
  };

  return (
    <>
      <TitleInput
        value={editedName}
        onChange={setEditedName}
        onConfirm={handleConfirm}
        onCancel={handleCancel}
        disabled={pending}
        inputRef={inputRef}
      />

      <ButtonsContainer>
        <StatusIndicator error={error} pending={pending} />
        <IconButton
          icon={Save}
          title="Save"
          disabled={pending}
          onClick={handleSave}
        />
        <IconButton
          icon={CopyPlus}
          title="Duplicate"
          disabled={pending}
          onClick={() => {
            setMode("duplicating");
            const name = `${fileName.replace(/\.xlsx$/, "")} (copy).xlsx`;
            setEditedName(name);
            requestAnimationFrame(() => {
              inputRef.current?.focus();
              inputRef.current?.setSelectionRange(0, name.lastIndexOf(".xlsx"));
            });
          }}
        />
        <IconButton
          icon={Download}
          title="Download"
          onClick={() => downloadFile(filePath)}
        />
        <IconButton
          icon={X}
          title="Close"
          onClick={() => navigateToFolder(filePath)}
        />
      </ButtonsContainer>
    </>
  );
}

export default function TitleBar({
  onSave,
  onCreate,
  onRename,
  location,
}: {
  onSave: () => Promise<void>;
  onCreate: (name: string) => Promise<void>;
  onRename: (name: string) => Promise<void>;
  location?: Location;
}) {
  return (
    <div
      css={{
        height: 48,
        minHeight: 48,
        background: "#fff",
        borderBottom: "1px solid #e0e0e0",
        display: "flex",
        alignItems: "center",
        position: "relative",
      }}
    >
      {location ? (
        <ExistingFileTitleBar
          onSave={onSave}
          onCreate={onCreate}
          onRename={onRename}
          location={location}
        />
      ) : (
        <NewFileTitleBar onCreate={onCreate} />
      )}
    </div>
  );
}
