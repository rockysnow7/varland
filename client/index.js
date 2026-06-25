const API = "http://127.0.0.1:3000";

const MIN_COLS = 10;
const MIN_ROWS = 10;
const SCROLL_BUFFER = 10;

const state = {
    raw: new Map(),
    evaluated: [],
    visibleRange: { startCol: 0, startRow: 0, endCol: MIN_COLS - 1, endRow: MIN_ROWS - 1 },
    errors: new Map(),
    selected: null,
    numCols: MIN_COLS,
    numRows: MIN_ROWS,
};

async function getNumCols() {
    const res = await fetch(`${API}/api/get/num-cols`, {
        method: "GET",
    });
    return res.json();
}

async function getNumRows() {
    const res = await fetch(`${API}/api/get/num-rows`, {
        method: "GET",
    });
    return res.json();
}

async function getRaw(col, row) {
    const res = await fetch(`${API}/api/get/raw`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ col, row }),
    });
    return res.json();
}

async function getEvaluatedRange(startCol, startRow, endCol, endRow) {
    const res = await fetch(`${API}/api/get/evaluated-range`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
            start_col: startCol,
            start_row: startRow,
            end_col: endCol,
            end_row: endRow,
        }),
    });
    if (!res.ok) return [];
    return res.json();
}

async function setRawValue(col, row, value) {
    const res = await fetch(`${API}/api/set/raw`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ col, row, value }),
    });
    return res.ok;
}

function cellKey(col, row) {
    return `${col},${row}`;
}

async function parseInput(text) {
    const res = await fetch(`${API}/api/parse`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(text),
    });
    return res.json();
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

function valueToString(value, inner = false) {
    if (value == null || value === "Null") return "";
    if (typeof value !== "object") return String(value);
    if ("Bool" in value) return String(value.Bool);
    if ("Int" in value) return String(value.Int);
    if ("Float" in value) return String(value.Float);
    if ("String" in value) {
        if (inner) return `"${value.String}"`;
        return value.String;
    }
    if ("List" in value) {
        return `[${value.List.map(v => valueToString(v, true)).join(", ")}]`;
    }
    if ("FunctionCall" in value) {
        const { function_name, arguments: args } = value.FunctionCall;
        const eq = inner ? "" : "=";
        return `${eq}${function_name}(${args.map(v => valueToString(v, true)).join(", ")})`;
    }
    if ("CloneCell" in value) {
        const { col, row } = value.CloneCell;
        const eq = inner ? "" : "=";
        return `${eq}${colLabel(col)}${row + 1}`;
    }
    if ("CloneCellRange" in value) {
        const { start_col, start_row, end_col, end_row } = value.CloneCellRange;
        const eq = inner ? "" : "=";
        return `${eq}${colLabel(start_col)}${start_row + 1}:${colLabel(end_col)}${end_row + 1}`;
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

    const { startCol, startRow } = state.visibleRange;
    const relCol = col - startCol;
    const relRow = row - startRow;
    const evaluatedCell = state.evaluated[relCol]?.[relRow];

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
    return valueToString(state.raw.get(cellKey(col, row)));
}

function fullGridRange() {
    return {
        startCol: 0,
        startRow: 0,
        endCol: state.numCols - 1,
        endRow: state.numRows - 1,
    };
}

function getVisibleRange() {
    const container = document.getElementById("grid-container");
    const table = container?.querySelector("table");
    if (!container || !table) {
        return fullGridRange();
    }

    const sampleCell = table.querySelector("td[data-col]");
    if (!sampleCell) {
        return fullGridRange();
    }

    const cellW = sampleCell.offsetWidth;
    const cellH = sampleCell.offsetHeight;
    if (!cellW || !cellH) {
        return fullGridRange();
    }

    const rowHeader = table.querySelector("tbody th")?.offsetWidth ?? 0;
    const colHeader = table.querySelector("thead tr")?.offsetHeight ?? 0;

    const startCol = Math.max(
        0,
        Math.floor((container.scrollLeft - rowHeader) / cellW) - SCROLL_BUFFER,
    );
    const startRow = Math.max(
        0,
        Math.floor((container.scrollTop - colHeader) / cellH) - SCROLL_BUFFER,
    );
    const endCol = Math.min(
        state.numCols - 1,
        Math.ceil((container.scrollLeft - rowHeader + container.clientWidth) / cellW) + SCROLL_BUFFER,
    );
    const endRow = Math.min(
        state.numRows - 1,
        Math.ceil((container.scrollTop - colHeader + container.clientHeight) / cellH) + SCROLL_BUFFER,
    );

    if (endCol < startCol || endRow < startRow) {
        return fullGridRange();
    }

    return { startCol, startRow, endCol, endRow };
}

function rangesEqual(a, b) {
    return a.startCol === b.startCol
        && a.startRow === b.startRow
        && a.endCol === b.endCol
        && a.endRow === b.endRow;
}

function render() {
    const app = document.getElementById("app");
    const numCols = state.numCols;
    const numRows = state.numRows;

    let html = `<div class="formula-bar">
        <span class="formula-bar-label">${escapeHtml(selectedLabel())}</span>
        <input id="formula-bar-input" type="text" value="${escapeHtml(selectedRawText())}"${state.selected ? "" : " disabled"}>
    </div>`;

    html += `<div class="grid-container" id="grid-container" style="overflow:auto;height:calc(100vh - 48px);">`;
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

    html += "</tbody></table></div>";
    app.innerHTML = html;
}

async function selectCell(col, row) {
    state.numCols = Math.max(state.numCols, col + 1, MIN_COLS);
    state.numRows = Math.max(state.numRows, row + 1, MIN_ROWS);
    state.selected = { col, row };

    const key = cellKey(col, row);
    if (!state.raw.has(key)) {
        state.raw.set(key, await getRaw(col, row));
    }

    render();
    attachHandlers();
}

async function commitFormulaBar() {
    if (!state.selected) return;

    const input = document.getElementById("formula-bar-input");
    if (!input) return;

    const { col, row } = state.selected;
    const key = cellKey(col, row);
    const result = await parseInput(input.value);

    if ("Err" in result) {
        state.errors.set(key, result.Err);
        render();
        attachHandlers();
        return;
    }

    const ok = await setRawValue(col, row, result.Ok);
    if (!ok) {
        state.errors.set(key, "Failed to set value");
    } else {
        state.errors.delete(key);
        state.raw.set(key, result.Ok);
    }
    await refresh();
}

async function refresh() {
    const container = document.getElementById("grid-container");
    const scrollLeft = container?.scrollLeft ?? 0;
    const scrollTop = container?.scrollTop ?? 0;

    state.numCols = Math.max(state.numCols, await getNumCols());
    state.numRows = Math.max(state.numRows, await getNumRows());

    const range = getVisibleRange();
    state.visibleRange = range;
    state.evaluated = await getEvaluatedRange(
        range.startCol,
        range.startRow,
        range.endCol,
        range.endRow,
    );
    render();
    attachHandlers();

    const newContainer = document.getElementById("grid-container");
    if (newContainer) {
        newContainer.scrollLeft = scrollLeft;
        newContainer.scrollTop = scrollTop;
    }
}

let scrollTimeout;
function onGridScroll() {
    clearTimeout(scrollTimeout);
    scrollTimeout = setTimeout(async () => {
        const range = getVisibleRange();
        if (rangesEqual(range, state.visibleRange)) return;
        await refresh();
    }, 100);
}

async function moveCursorUp() {
    if (!state.selected) return;
    const { col, row } = state.selected;
    if (row === 0) return;
    await selectCell(col, row - 1);
}

async function moveCursorDown() {
    if (!state.selected) return;
    const { col, row } = state.selected;
    if (row === state.numRows - 1) return;
    await selectCell(col, row + 1);
}

async function moveCursorLeft() {
    if (!state.selected) return;
    const { col, row } = state.selected;
    if (col === 0) return;
    await selectCell(col - 1, row);
}

async function moveCursorRight() {
    if (!state.selected) return;
    const { col, row } = state.selected;
    if (col === state.numCols - 1) return;
    await selectCell(col + 1, row);
}

function attachHandlers() {
    document.querySelectorAll("td[data-col]").forEach(td => {
        td.addEventListener("click", () => {
            selectCell(+td.dataset.col, +td.dataset.row);
        });
    });

    const gridContainer = document.getElementById("grid-container");
    if (gridContainer) {
        gridContainer.addEventListener("scrollend", onGridScroll);
    }

    const formulaBar = document.getElementById("formula-bar-input");
    if (!formulaBar) return;

    formulaBar.addEventListener("keydown", async (e) => {
        switch (e.key) {
            case "Enter":
                e.preventDefault();
                commitFormulaBar();
                break;
            case "Escape":
                e.preventDefault();
                formulaBar.value = "";
                render();
                break;
            default:
                break;
        }
    });

    formulaBar.addEventListener("change", () => {
        commitFormulaBar();
    });
}

refresh();

async function deleteCell() {
    if (!state.selected) return;
    const { col, row } = state.selected;
    await setRawValue(col, row, { "Null": null });
    state.raw.delete(cellKey(col, row));
    await refresh();
}

function isVisibleChar(key) {
    return key.length === 1 && key.match(/[a-zA-Z0-9`~!@£$%^&*()_+\-=\[\]{};:'",.<>\/?]/);
}

document.addEventListener("keydown", async (e) => {
    const formulaBar = document.getElementById("formula-bar-input");
    if (!formulaBar || document.activeElement === formulaBar) return;

    switch (e.key) {
        case "Enter":
            e.preventDefault();
            formulaBar.focus();
            break;
        case "ArrowUp":
            e.preventDefault();
            await moveCursorUp();
            break;
        case "ArrowDown":
            e.preventDefault();
            await moveCursorDown();
            break;
        case "ArrowLeft":
            e.preventDefault();
            await moveCursorLeft();
            break;
        case "ArrowRight":
            e.preventDefault();
            await moveCursorRight();
            break;
        case "Backspace":
            e.preventDefault();
            await deleteCell();
            break;
    }

    if (isVisibleChar(e.key)) {
        if (formulaBar.value !== "") {
            formulaBar.value = "";
        }
        formulaBar.focus();
    }
});

window.addEventListener("resize", () => {
    clearTimeout(scrollTimeout);
    scrollTimeout = setTimeout(refresh, 100);
});
