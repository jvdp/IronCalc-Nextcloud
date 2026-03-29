import { ClickAwayListener, Popper } from "@mui/material";
import { ChevronDown, Download, Save, X } from "lucide-react";
import { useRef, useState } from "react";

const buttonStyles = {
  display: "inline-flex",
  width: 24,
  minWidth: 24,
  height: 24,
  padding: 0,
  alignItems: "center",
  justifyContent: "center",
  border: "none",
  borderRadius: 4,
  outline: "1px solid #fff",
  cursor: "pointer",
  backgroundColor: "#fff",
  color: "#333",
  "&:hover": {
    outlineColor: "#e0e0e0",
  },
  "&:active": {
    backgroundColor: "#e0e0e0",
  },
} as const;

function SaveButtonGroup() {
  const [open, setOpen] = useState(false);
  const anchorRef = useRef<HTMLButtonElement>(null);

  return (
    <ClickAwayListener onClickAway={() => setOpen(false)}>
      <div css={{ display: "inline-flex" }}>
        <div
          css={{
            display: "flex",
            borderRadius: 4,
            outline: "1px solid #fff",
            "&:hover": { outlineColor: "#e0e0e0" },
          }}
        >
          <button
            type="button"
            css={{
              ...buttonStyles,
              outline: "none",
              borderTopRightRadius: 0,
              borderBottomRightRadius: 0,
            }}
          >
            <Save size={16} />
          </button>
          <button
            type="button"
            ref={anchorRef}
            onClick={() => setOpen((v) => !v)}
            css={{
              ...buttonStyles,
              outline: "none",
              width: 16,
              minWidth: 16,
              borderTopLeftRadius: 0,
              borderBottomLeftRadius: 0,
              borderLeft: "1px solid #e0e0e0",
            }}
          >
            <ChevronDown size={12} />
          </button>
        </div>
        <Popper
          open={open}
          anchorEl={anchorRef.current}
          placement="bottom-end"
          modifiers={[{ name: "offset", options: { offset: [0, 4] } }]}
          style={{ zIndex: 1300 }}
        >
          <div
            css={{
              backgroundColor: "#fff",
              border: "1px solid #e0e0e0",
              borderRadius: 4,
              boxShadow: "0 2px 8px rgba(0,0,0,0.15)",
              whiteSpace: "nowrap",
            }}
          >
            <button
              type="button"
              onClick={() => setOpen(false)}
              css={{
                display: "block",
                width: "100%",
                padding: "6px 12px",
                border: "none",
                background: "none",
                cursor: "pointer",
                fontSize: 13,
                textAlign: "left",
                color: "#333",
                "&:hover": {
                  backgroundColor: "#f5f5f5",
                },
              }}
            >
              Save As…
            </button>
          </div>
        </Popper>
      </div>
    </ClickAwayListener>
  );
}

export default function TitleBar() {
  const fileName = window.location.pathname.split("/").at(-1);
  return (
    <div
      css={{
        height: 24,
        minHeight: 24,
        padding: 12,
        background: "#fff",
        color: "#000",
        display: "flex",
        gap: 24,
        alignItems: "center",
        borderBottom: "1px solid #e0e0e0",
        justifyContent: "space-between",
      }}
    >
      <div css={{ width: 100 }}></div>
      <div css={{ flex: 1, textAlign: "center", fontWeight: "bold" }}>
        {fileName}
      </div>
      <div
        css={{
          width: 100,
          textAlign: "right",
          display: "flex",
          justifyContent: "flex-end",
          gap: 10,
        }}
      >
        <SaveButtonGroup />
        <button type="button" css={buttonStyles}>
          <Download size={16} />
        </button>
        <button type="button" css={buttonStyles}>
          <X size={16} />
        </button>
      </div>
    </div>
  );
}
