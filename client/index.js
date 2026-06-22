const API = "http://127.0.0.1:3000";

const state = {
    raw: [],
    evaluated: [],
    errors: new Map(),
    selected: null,
};

async function getRawState() {
    const res = await fetch(`${API}/api/get-raw-state`);
    return res.json();
}

async function getEvaluatedState() {
    const res = await fetch(`${API}/api/get-evaluated-state`);
    return res.json();
}

async function setRawValue(col, row, value) {
    const res = await fetch(`${API}/api/set-raw-value`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ col, row, value }),
    });
    return res.ok;
}

function cellKey(col, row) {
    return `${col},${row}`;
}

function parseInput(text) {
    if (text.trim() === "") return "Null";
    if (/^-?\d+$/.test(text)) return { Int: parseInt(text, 10) };
    if (/^-?\d+\.\d+$/.test(text)) return { Float: parseFloat(text) };
    return { String: text };
}

function colLabel(i) {
    let s = "", n = i + 1;
    while (n > 0) { n--; s = String.fromCharCode(65 + (n % 26)) + s; n = Math.floor(n / 26); }
    return s;
}

function escapeHtml(text) {
    return String(text)
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;");
}

function isErrorCell(cell) {
    return cell != null && typeof cell === "object" && "Err" in cell;
}

function valueToString(value) {
    if (value == null || value === "Null") return "";
    if (typeof value !== "object") return String(value);
    if ("Bool" in value) return String(value.Bool);
    if ("Int" in value) return String(value.Int);
    if ("Float" in value) return String(value.Float);
    if ("String" in value) return value.String;
    if ("List" in value) {
        return `[${value.List.map(valueToString).join(", ")}]`;
    }
    if ("FunctionCall" in value) {
        const { function_name, arguments: args } = value.FunctionCall;
        return `${function_name}(${args.map(valueToString).join(", ")})`;
    }
    if ("CloneCell" in value) {
        const { col, row } = value.CloneCell;
        return `${colLabel(col)}${row}`;
    }
    if ("CloneCellRange" in value) {
        const { start_col, start_row, end_col, end_row } = value.CloneCellRange;
        return `${colLabel(start_col)}${start_row}:${colLabel(end_col)}${end_row}`;
    }
    return "";
}

function displayValue(cell) {
    if (!cell) return "";
    if ("Ok" in cell) return valueToString(cell.Ok);
    if ("Err" in cell) return cell.Err;
    return "";
}

function cellDisplay(col, row) {
    const key = cellKey(col, row);
    const clientError = state.errors.get(key);
    if (clientError) {
        return { text: clientError, isError: true };
    }

    const evaluatedCell = state.evaluated[col]?.[row];
    return {
        text: displayValue(evaluatedCell),
        isError: isErrorCell(evaluatedCell),
    };
}

function selectedLabel() {
    if (!state.selected) return "";
    return `${colLabel(state.selected.col)}${state.selected.row + 1}`;
}

function selectedRawText() {
    if (!state.selected) return "";
    const { col, row } = state.selected;
    return valueToString(state.raw[col]?.[row]);
}

function render() {
    const app = document.getElementById("app");
    const { raw, evaluated } = state;

    const maxRows = Math.max(
        0,
        ...raw.map(c => c?.length ?? 0),
        ...evaluated.map(c => c?.length ?? 0),
    );
    const numCols = Math.max(10, raw.length, evaluated.length);
    const numRows = Math.max(10, maxRows);

    let html = `<div class="formula-bar">
        <span class="formula-bar-label">${escapeHtml(selectedLabel())}</span>
        <input id="formula-bar-input" type="text" value="${escapeHtml(selectedRawText())}"${state.selected ? "" : " disabled"}>
    </div>`;

    html += "<table><thead><tr><th></th>";

    for (let c = 0; c < numCols; c++) {
        html += `<th>${escapeHtml(colLabel(c))}</th>`;
    }
    html += "</tr></thead><tbody>";

    for (let r = 0; r < numRows; r++) {
        html += `<tr><th>${r + 1}</th>`;
        for (let c = 0; c < numCols; c++) {
            const { text: displayText, isError } = cellDisplay(c, r);
            const isSelected = state.selected?.col === c && state.selected?.row === r;
            const classes = [
                isError ? "error" : "",
                isSelected ? "selected" : "",
            ].filter(Boolean).join(" ");
            html += `<td class="${classes}" data-col="${c}" data-row="${r}">${escapeHtml(displayText)}</td>`;
        }
        html += "</tr>";
    }

    html += "</tbody></table>";
    app.innerHTML = html;
}

function selectCell(col, row) {
    state.selected = { col, row };
    render();
    attachHandlers();
    document.getElementById("formula-bar-input")?.focus();
}

async function commitFormulaBar() {
    if (!state.selected) return;

    const input = document.getElementById("formula-bar-input");
    if (!input) return;

    const { col, row } = state.selected;
    const value = parseInput(input.value);
    const key = cellKey(col, row);
    const ok = await setRawValue(col, row, value);
    if (!ok) {
        state.errors.set(key, "Failed to set value");
    } else {
        state.errors.delete(key);
    }
    await refresh();
}

async function refresh() {
    const [raw, evaluated] = await Promise.all([
        getRawState(),
        getEvaluatedState(),
    ]);
    state.raw = raw;
    state.evaluated = evaluated;
    render();
    attachHandlers();
}

function attachHandlers() {
    document.querySelectorAll("td[data-col]").forEach(td => {
        td.addEventListener("click", () => {
            selectCell(+td.dataset.col, +td.dataset.row);
        });
    });

    const formulaBar = document.getElementById("formula-bar-input");
    if (!formulaBar) return;

    formulaBar.addEventListener("keydown", (e) => {
        if (e.key === "Enter") {
            e.preventDefault();
            commitFormulaBar();
        }
    });

    formulaBar.addEventListener("change", () => {
        commitFormulaBar();
    });
}

refresh();
