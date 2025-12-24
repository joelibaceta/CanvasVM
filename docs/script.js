// Wait for DOM to be ready
document.addEventListener('DOMContentLoaded', async function() {

// Load WASM module
let wasm;
try {
    const wasmModule = await import('./pkg/canvas_wasm.js');
    await wasmModule.default();
    wasm = wasmModule;
    logToTerminal('✓ WASM module loaded successfully', 'output');
} catch (error) {
    logToTerminal(`✗ Failed to load WASM: ${error.message}`, 'error');
    logToTerminal('Running in mock mode', 'output');
}

// State
let vm = null;
let debugger_ = null; // PietDebugger instance for debug mode
let program = null;
let currentImage = null;
let executionStats = {
    startTime: 0,
    endTime: 0,
    steps: 0,
    running: false
};

// Status bar elements
const statusIndicator = document.getElementById('status-indicator');
const statusText = document.getElementById('status-text');
const stepsCount = document.getElementById('steps-count');
const execTime = document.getElementById('exec-time');
const execSpeed = document.getElementById('exec-speed');
const ipDisplay = document.getElementById('ip-display');
const posDisplay = document.getElementById('pos-display');
const modeBadge = document.getElementById('mode-badge');
const dpDisplay = document.getElementById('dp-display');
const ccDisplay = document.getElementById('cc-display');

// Update status bar
function updateStatus(status, state = 'idle') {
    if (statusText) statusText.textContent = status;
    if (statusIndicator) {
        statusIndicator.className = 'w-2 h-2 rounded-full';
        switch(state) {
            case 'running':
                statusIndicator.classList.add('bg-green-500', 'animate-pulse');
                if (modeBadge) {
                    modeBadge.textContent = 'RUNNING';
                    modeBadge.className = 'px-2 py-0.5 bg-green-500 text-white text-[10px] font-bold';
                }
                break;
            case 'halted':
                statusIndicator.classList.add('bg-mondrian-red');
                if (modeBadge) {
                    modeBadge.textContent = 'HALTED';
                    modeBadge.className = 'px-2 py-0.5 bg-mondrian-red text-white text-[10px] font-bold';
                }
                break;
            case 'debug':
                statusIndicator.classList.add('bg-mondrian-yellow');
                if (modeBadge) {
                    modeBadge.textContent = 'DEBUG';
                    modeBadge.className = 'px-2 py-0.5 bg-mondrian-yellow text-black text-[10px] font-bold';
                }
                break;
            default:
                statusIndicator.classList.add('bg-gray-400');
                if (modeBadge) {
                    modeBadge.textContent = 'IDLE';
                    modeBadge.className = 'px-2 py-0.5 bg-gray-200 text-gray-600 text-[10px] font-bold';
                }
        }
    }
}

function updateExecutionStats(steps, timeMs) {
    if (stepsCount) stepsCount.textContent = steps.toLocaleString();
    if (execTime) {
        if (timeMs < 1) {
            execTime.textContent = `${(timeMs * 1000).toFixed(2)}µs`;
        } else if (timeMs < 1000) {
            execTime.textContent = `${timeMs.toFixed(2)}ms`;
        } else {
            execTime.textContent = `${(timeMs / 1000).toFixed(2)}s`;
        }
    }
    if (execSpeed && timeMs > 0) {
        const stepsPerSec = (steps / timeMs) * 1000;
        if (stepsPerSec >= 1000000) {
            execSpeed.textContent = `${(stepsPerSec / 1000000).toFixed(2)}M ops/s`;
            execSpeed.className = 'font-bold text-green-600';
        } else if (stepsPerSec >= 1000) {
            execSpeed.textContent = `${(stepsPerSec / 1000).toFixed(2)}K ops/s`;
            execSpeed.className = 'font-bold text-green-600';
        } else {
            execSpeed.textContent = `${stepsPerSec.toFixed(0)} ops/s`;
            execSpeed.className = 'font-bold text-gray-600';
        }
    }
}

function updatePositionDisplay(x, y, ip) {
    if (posDisplay) posDisplay.textContent = `(${x},${y})`;
    if (ipDisplay) ipDisplay.textContent = `0x${ip.toString(16).toUpperCase().padStart(2, '0')}`;
}

function updateDpCc(dp, cc) {
    if (dpDisplay) dpDisplay.textContent = dp.toUpperCase();
    if (ccDisplay) ccDisplay.textContent = cc.toUpperCase();
}

// Running visual effect
const bytecodePanel = document.querySelector('.lg\\:col-span-3.bg-white');
const canvasPreview = document.getElementById('canvas-preview');

function setRunningEffect(isRunning) {
    if (isRunning) {
        // Add gray overlay effect
        if (bytecodePanel) bytecodePanel.classList.add('running-overlay', 'bytecode-panel');
        if (canvasPreview) {
            canvasPreview.classList.add('canvas-running');
            canvasPreview.classList.remove('canvas-idle');
        }
    } else {
        // Remove effect
        if (bytecodePanel) bytecodePanel.classList.remove('running-overlay', 'bytecode-panel');
        if (canvasPreview) {
            canvasPreview.classList.remove('canvas-running');
            canvasPreview.classList.add('canvas-idle');
        }
    }
}

// File tab elements and functions
const fileTabBar = document.getElementById('file-tab-bar');
const fileTabName = document.getElementById('file-tab-name');

function showFileTab(filename) {
    if (fileTabBar) fileTabBar.style.display = 'flex';
    if (fileTabName) fileTabName.textContent = filename || 'untitled.png';
}

function hideFileTab() {
    if (fileTabBar) fileTabBar.style.display = 'none';
}

// Input Modal
const inputModal = document.getElementById('input-modal');
const inputModalIcon = document.getElementById('input-modal-icon');
const inputModalPrompt = document.getElementById('input-modal-prompt');
const inputModalField = document.getElementById('input-modal-field');
const inputModalError = document.getElementById('input-modal-error');
const inputModalSubmit = document.getElementById('input-modal-submit');
const inputModalCancel = document.getElementById('input-modal-cancel');

let inputModalCallback = null;
let inputModalType = null;

function showInputModal(type, callback) {
    console.log('showInputModal called with type:', type);
    
    // Use browser prompt as simple solution
    const promptText = type === 'number' 
        ? 'Enter a number (Cancel or empty to stop program):' 
        : 'Enter a single character (Cancel or empty to stop program):';
    const result = prompt(promptText);
    
    console.log('User entered:', result);
    
    // Treat Cancel (null) or empty string as EOF/end-of-input
    if (result === null || result === '') {
        console.log('User cancelled input or entered empty - stopping execution (EOF)');
        // Signal that input was cancelled (callback with null)
        if (callback) callback(null);
        return;
    }
    
    if (callback) {
        if (type === 'number') {
            const num = parseInt(result, 10);
            console.log('Providing number input:', num);
            callback(isNaN(num) ? 0 : num);
        } else {
            const charCode = result.charCodeAt(0);
            console.log('Providing char input:', charCode, '(' + result + ')');
            callback(charCode);
        }
    }
}

function hideInputModal() {
    if (inputModal) inputModal.style.display = 'none';
    inputModalCallback = null;
    inputModalType = null;
}

function submitInput() {
    // Not used with prompt() fallback
}

// Modal event listeners
if (inputModalSubmit) {
    inputModalSubmit.addEventListener('click', submitInput);
}

if (inputModalCancel) {
    inputModalCancel.addEventListener('click', () => {
        hideInputModal();
        setRunningEffect(false);
        updateStatus('Cancelled', 'idle');
        logToTerminal('⚠ Execution cancelled by user', 'output');
    });
}

if (inputModalField) {
    inputModalField.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') {
            submitInput();
        } else if (e.key === 'Escape') {
            hideInputModal();
            setRunningEffect(false);
            updateStatus('Cancelled', 'idle');
            logToTerminal('⚠ Execution cancelled by user', 'output');
        }
    });
}

// Terminal
function logToTerminal(message, type = 'output') {
    const terminal = document.getElementById('terminal-output');
    const line = document.createElement('div');
    
    if (type === 'prompt') {
        line.className = 'text-black font-bold mb-1';
        line.textContent = `> ${message}`;
    } else if (type === 'error') {
        line.className = 'text-red-600 mb-1';
        line.textContent = message;
    } else if (type === 'program-output') {
        line.className = 'text-green-600 font-bold mb-1 bg-green-50 px-2 py-1 rounded';
        line.textContent = message;
    } else {
        line.className = 'text-gray-400 mb-1';
        line.textContent = message;
    }
    
    terminal.appendChild(line);
    terminal.scrollTop = terminal.scrollHeight;
}

window.clearTerminal = function() {
    document.getElementById('terminal-output').innerHTML = `
        <div class="text-gray-400 mb-1">Terminal cleared.</div>
        <div class="text-black font-bold">&gt; _</div>
    `;
};

// Get selected codel size (0 = auto)
function getSelectedCodelSize() {
    const select = document.getElementById('codel-size-select');
    return select ? parseInt(select.value, 10) : 0;
}

// Get watchdog limit from selector (0 = disabled)
function getWatchdogLimit() {
    const select = document.getElementById('watchdog-select');
    return select ? parseInt(select.value, 10) : 100000;
}

// Apply watchdog setting to VM
function applyWatchdogSetting() {
    if (vm) {
        const limit = getWatchdogLimit();
        vm.set_max_steps(limit);
        if (limit === 0) {
            logToTerminal('⚠️ Watchdog disabled - infinite loops possible', 'output');
        } else {
            logToTerminal(`⏱️ Watchdog set to ${limit.toLocaleString()} steps`, 'output');
        }
    }
}

// Compile image to bytecode
function compileImageToBytecode(imageData) {
    logToTerminal('Loading image into VM...', 'prompt');
    
    if (!wasm) {
        logToTerminal('✗ WASM not loaded, cannot compile', 'error');
        return null;
    }
    
    try {
        // Create Canvas (VM wrapper) instance
        if (!vm) {
            vm = new wasm.Canvas();
        }
        
        // Apply watchdog setting before loading
        const watchdogLimit = getWatchdogLimit();
        vm.set_max_steps(watchdogLimit);
        
        // Load image into VM
        const width = imageData.width;
        const height = imageData.height;
        const rgbaData = new Uint8Array(imageData.data);
        
        // Get codel size from selector
        const codelSize = getSelectedCodelSize();
        
        // Detect codel size if auto
        if (codelSize === 0) {
            const detectedSize = wasm.Canvas.detect_codel_size(rgbaData, width, height);
            logToTerminal(`🔍 Auto-detected codel size: ${detectedSize}px`, 'output');
        }
        
        // Load with specified or auto codel size
        vm.paint_with_codel_size(rgbaData, width, height, codelSize);
        
        const effectiveSize = codelSize === 0 ? 
            wasm.Canvas.detect_codel_size(rgbaData, width, height) : codelSize;
        const gridWidth = Math.floor(width / effectiveSize);
        const gridHeight = Math.floor(height / effectiveSize);
        
        logToTerminal(`✓ Program loaded: ${width}x${height}px → ${gridWidth}x${gridHeight} codels`, 'output');
        
        // Compile to bytecode
        try {
            const bytecode = vm.compile_to_bytecode();
            if (bytecode && bytecode.length > 0) {
                program = bytecode;
                updateBytecodeTable(bytecode);
                logToTerminal(`✓ Compiled ${bytecode.length} instructions`, 'output');
            } else {
                logToTerminal('ℹ Bytecode compilation returned empty (feature in progress)', 'prompt');
                program = [];
                updateBytecodeTable([]);
            }
        } catch (compileError) {
            logToTerminal(`ℹ Bytecode compilation not fully implemented yet`, 'prompt');
            program = [];
            updateBytecodeTable([]);
        }
        
        logToTerminal('VM ready to execute', 'output');
        
        return program;
    } catch (error) {
        logToTerminal(`✗ Failed to load program: ${error.message}`, 'error');
        console.error('Load error:', error);
        return null;
    }
}

// Update bytecode table in UI
function updateBytecodeTable(bytecode) {
    const tbody = document.querySelector('#bytecode-table tbody');
    if (!tbody) return;
    
    tbody.innerHTML = '';
    
    const colorMap = {
        'Push': 'text-mondrian-blue',
        'Add': 'text-black',
        'Subtract': 'text-mondrian-red',
        'Multiply': 'text-mondrian-red',
        'Divide': 'text-gray-600',
        'Mod': 'text-gray-600',
        'Duplicate': 'text-mondrian-blue',
        'OutNumber': 'text-gray-600',
        'OutChar': 'text-gray-600',
        'Halt': 'text-black'
    };
    
    bytecode.forEach((instr, index) => {
        const row = document.createElement('tr');
        row.className = 'group hover:bg-blue-50';
        row.dataset.index = index;
        
        // Extract opcode and value from the BytecodeInstruction object
        let opName = instr.opcode || 'UNKNOWN';
        let value = instr.to_color || '-';
        
        const colorClass = colorMap[opName] || 'text-gray-600';
        const isBlack = colorClass.includes('bg-black');
        
        if (isBlack) {
            row.className = 'bg-black text-white';
        }
        
        row.innerHTML = `
            <td class="py-2 pl-4 text-gray-400 group-hover:text-black">${index.toString(16).toUpperCase().padStart(2, '0')}</td>
            <td class="py-2 font-bold ${colorClass}">${opName.toUpperCase()}</td>
            <td class="py-2 pr-4 text-right font-medium ${value === '-' ? 'opacity-50' : ''}">${value}</td>
        `;
        
        tbody.appendChild(row);
    });
}

// Update stack display
function updateStackDisplay(stack) {
    const stackDisplay = document.getElementById('stack-display');
    if (!stackDisplay) return;
    
    if (stack.length === 0) {
        stackDisplay.innerHTML = '<span class="text-xs font-mono text-gray-400">[ empty ]</span>';
        return;
    }
    
    // Show last 8 items (top of stack on the right)
    const displayStack = stack.slice(-8);
    stackDisplay.innerHTML = displayStack.map((val, idx) => {
        const isTop = idx === displayStack.length - 1;
        return `<span class="px-1.5 py-0.5 text-xs font-mono font-bold border ${
            isTop ? 'bg-mondrian-yellow border-black' : 'bg-gray-100 border-gray-300'
        }">${val}</span>`;
    }).join('');
    
    if (stack.length > 8) {
        stackDisplay.innerHTML = '<span class="text-xs text-gray-400 mr-1">...</span>' + stackDisplay.innerHTML;
    }
}

// Highlight current instruction in bytecode table
function highlightBytecodeRow(index) {
    const tbody = document.querySelector('#bytecode-table tbody');
    if (!tbody) return;
    
    // Remove previous highlight
    tbody.querySelectorAll('tr').forEach(row => {
        row.classList.remove('!bg-mondrian-yellow', '!bg-opacity-30', 'ring-2', 'ring-black', 'ring-inset');
    });
    
    // Add highlight to current row
    const currentRow = tbody.querySelector(`tr[data-index="${index}"]`);
    if (currentRow) {
        currentRow.classList.add('!bg-mondrian-yellow', '!bg-opacity-30', 'ring-2', 'ring-black', 'ring-inset');
        currentRow.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
    }
}

// File upload
const fileInput = document.getElementById('file-input');
const canvas = document.getElementById('canvas-preview');
const uploadArea = document.getElementById('upload-area');

// Store original image for redrawing with highlights
let originalImageData = null;
let grayscaleImageData = null;
let currentCodelSize = 1;
let visitedCodels = new Set(); // Track visited codels for debug visualization
let debugMode = false;

// Convert image to grayscale
function createGrayscaleImage(imageData) {
    const data = new Uint8ClampedArray(imageData.data);
    for (let i = 0; i < data.length; i += 4) {
        const gray = Math.round(data[i] * 0.299 + data[i + 1] * 0.587 + data[i + 2] * 0.114);
        data[i] = gray;     // R
        data[i + 1] = gray; // G
        data[i + 2] = gray; // B
        // Alpha stays the same
    }
    return new ImageData(data, imageData.width, imageData.height);
}

// Start debug mode - convert image to grayscale
function startDebugVisualization(codelSize) {
    if (!canvas || !originalImageData) {
        console.log('startDebugVisualization: canvas or originalImageData missing');
        return;
    }
    
    debugMode = true;
    currentCodelSize = codelSize;
    visitedCodels.clear();
    
    // Create grayscale version
    grayscaleImageData = createGrayscaleImage(originalImageData);
    
    // Draw grayscale image
    const ctx = canvas.getContext('2d');
    ctx.putImageData(grayscaleImageData, 0, 0);
    console.log('Debug mode started, codel size:', codelSize);
}

// End debug mode - restore original image
function endDebugVisualization() {
    debugMode = false;
    visitedCodels.clear();
    grayscaleImageData = null;
    
    if (canvas && originalImageData) {
        const ctx = canvas.getContext('2d');
        ctx.putImageData(originalImageData, 0, 0);
    }
    console.log('Debug mode ended');
}

// Get the average color of a codel from original image
function getCodelColor(pixelX, pixelY, codelSize) {
    if (!originalImageData) return { r: 128, g: 128, b: 128 };
    
    // Get color from center of codel
    const centerX = Math.min(pixelX + Math.floor(codelSize / 2), originalImageData.width - 1);
    const centerY = Math.min(pixelY + Math.floor(codelSize / 2), originalImageData.height - 1);
    const idx = (centerY * originalImageData.width + centerX) * 4;
    
    return {
        r: originalImageData.data[idx],
        g: originalImageData.data[idx + 1],
        b: originalImageData.data[idx + 2]
    };
}

// Draw debug visualization with visited codels colored and current highlighted
function drawDebugVisualization(currentPixelX, currentPixelY, codelSize) {
    if (!canvas || !originalImageData) {
        console.log('drawDebugVisualization: missing canvas or originalImageData');
        return;
    }
    
    const ctx = canvas.getContext('2d');
    
    // If grayscale not created yet, create it
    if (!grayscaleImageData) {
        grayscaleImageData = createGrayscaleImage(originalImageData);
    }
    
    // Start with grayscale base
    ctx.putImageData(grayscaleImageData, 0, 0);
    
    // Add current codel to visited FIRST
    const currentKey = `${currentPixelX},${currentPixelY}`;
    visitedCodels.add(currentKey);
    
    console.log(`Drawing ${visitedCodels.size} visited codels, current: (${currentPixelX}, ${currentPixelY}), codelSize: ${codelSize}`);
    
    // Draw all visited codels with their original colors
    visitedCodels.forEach(key => {
        const [px, py] = key.split(',').map(Number);
        const color = getCodelColor(px, py, codelSize);
        ctx.fillStyle = `rgb(${color.r}, ${color.g}, ${color.b})`;
        ctx.fillRect(px, py, codelSize, codelSize);
    });
    
    // Draw highlight rectangle around current codel
    ctx.strokeStyle = '#ff0000';
    ctx.lineWidth = Math.max(2, codelSize / 5);
    ctx.strokeRect(currentPixelX, currentPixelY, codelSize, codelSize);
    
    // Draw inner yellow border for visibility
    ctx.strokeStyle = '#ffff00';
    ctx.lineWidth = Math.max(1, codelSize / 10);
    ctx.strokeRect(currentPixelX + 2, currentPixelY + 2, codelSize - 4, codelSize - 4);
}

// Legacy highlight function (for non-debug mode)
function highlightCodel(pixelX, pixelY, codelSize) {
    if (!canvas || !originalImageData) {
        console.log('highlightCodel: missing canvas or originalImageData');
        return;
    }
    
    console.log(`highlightCodel called: (${pixelX}, ${pixelY}), codelSize: ${codelSize}, debugMode: ${debugMode}`);
    
    // Always use debug visualization when stepping
    drawDebugVisualization(pixelX, pixelY, codelSize);
}

// Clear codel highlight (restore original image)
function clearCodelHighlight() {
    if (!canvas || !originalImageData) return;
    endDebugVisualization();
}

if (canvas && fileInput) {
    const ctx = canvas.getContext('2d');

    fileInput.addEventListener('change', (e) => {
        const file = e.target.files[0];
        if (file) loadImage(file);
    });

    function loadImage(file) {
        const reader = new FileReader();
        reader.onload = (e) => {
            const img = new Image();
            img.onload = () => {
                canvas.width = img.width;
                canvas.height = img.height;
                ctx.drawImage(img, 0, 0);
                currentImage = ctx.getImageData(0, 0, img.width, img.height);
                originalImageData = ctx.getImageData(0, 0, img.width, img.height);
                
                console.log('Image loaded, originalImageData set:', originalImageData.width, 'x', originalImageData.height);
                
                // Show canvas, hide upload area, show file tab
                canvas.style.display = 'block';
                if (uploadArea) uploadArea.style.display = 'none';
                showFileTab(file.name);
                
                logToTerminal(`✓ Loaded: ${img.width}x${img.height} (${img.width * img.height} codels)`, 'prompt');
                
                // Compile to bytecode automatically
                compileImageToBytecode(currentImage);
                
                logToTerminal('Ready to execute. Click Play to run or Step to debug.', 'output');
            };
            img.src = e.target.result;
        };
        reader.readAsDataURL(file);
    }
}

// Control buttons
const playButton = document.querySelector('button[title="Play"]');
const stepButton = document.querySelector('button[title="Step"]');
const runFastButton = document.getElementById('run-fast-btn');
const resetButton = document.getElementById('reset-btn');

// Reset button handler
if (resetButton) {
    resetButton.addEventListener('click', () => {
        if (!vm) {
            logToTerminal('✗ No program loaded.', 'error');
            return;
        }
        
        try {
            vm.reset();
            endDebugVisualization(); // End debug mode and restore original image
            updateStatus('Reset to initial state', 'idle');
            updateExecutionStats(0, 0);
            updatePositionDisplay(0, 0, 0);
            updateDpCc('Right', 'Left');
            updateStackDisplay([]);
            highlightBytecodeRow(0);
            logToTerminal('🔄 VM reset to initial state', 'output');
        } catch (e) {
            logToTerminal(`✗ Reset failed: ${e.message}`, 'error');
        }
    });
}

// Close tab button - close current image and allow loading another
const closeTabButton = document.getElementById('close-tab-btn');
if (closeTabButton) {
    closeTabButton.addEventListener('click', () => {
        // Reset all state
        vm = null;
        program = null;
        currentImage = null;
        originalImageData = null;
        grayscaleImageData = null;
        visitedCodels.clear();
        debugMode = false;
        
        // Hide canvas and tab, show upload area
        if (canvas) {
            canvas.style.display = 'none';
            const ctx = canvas.getContext('2d');
            ctx.clearRect(0, 0, canvas.width, canvas.height);
        }
        hideFileTab();
        if (uploadArea) {
            uploadArea.style.display = 'flex';
        }
        
        // Clear bytecode table
        const tbody = document.querySelector('#bytecode-table tbody');
        if (tbody) {
            tbody.innerHTML = `<tr>
                <td colspan="3" class="py-12 text-center text-gray-400 text-xs">
                    No program loaded. Upload a Piet image to compile.
                </td>
            </tr>`;
        }
        
        // Reset displays
        updateStatus('Ready', 'idle');
        updateExecutionStats(0, 0);
        updatePositionDisplay(0, 0, 0);
        updateDpCc('Right', 'Left');
        updateStackDisplay([]);
        
        // Clear file input so same file can be selected again
        if (fileInput) {
            fileInput.value = '';
        }
        
        logToTerminal('🗑️ Image closed. Upload a new Piet program to begin.', 'output');
    });
}

// Codel size selector - recompile when changed
const codelSizeSelect = document.getElementById('codel-size-select');
if (codelSizeSelect) {
    codelSizeSelect.addEventListener('change', () => {
        if (!currentImage) {
            return; // No image loaded, nothing to recompile
        }
        
        logToTerminal('🔄 Codel size changed, recompiling...', 'prompt');
        
        // Reset VM state
        if (vm) {
            vm.reset();
        }
        endDebugVisualization();
        
        // Recompile with new codel size
        const result = compileImageToBytecode(currentImage);
        if (result) {
            updateBytecodeTable(result.bytecode);
            updateStackDisplay([]);
            updateStatus('Recompiled with new codel size', 'idle');
            updateExecutionStats(0, 0);
            updatePositionDisplay(0, 0, 0);
            highlightBytecodeRow(0);
            logToTerminal('✓ Recompilation complete. Ready to execute.', 'output');
        }
    });
}

// Watchdog selector - update limit when changed
const watchdogSelect = document.getElementById('watchdog-select');
if (watchdogSelect) {
    watchdogSelect.addEventListener('change', () => {
        applyWatchdogSetting();
    });
}

// Run Fast - execute all at once without animation (native speed)
if (runFastButton) {
    runFastButton.addEventListener('click', () => {
        if (!vm) {
            logToTerminal('✗ No program loaded. Upload a Piet image first.', 'error');
            return;
        }
        
        logToTerminal('⚡ Running at native speed...', 'prompt');
        updateStatus('Running (native)', 'running');
        setRunningEffect(true);
        
        // Use setTimeout to allow UI to update before blocking execution
        setTimeout(() => {
            try {
                // Reset first
                vm.reset();
                clearCodelHighlight();
                
                const maxSteps = 1000000; // 1M steps max
                const startTime = performance.now();
                let totalSteps = 0;
                
                const runBatch = () => {
                    // Check if input is needed before running
                    const needsInput = vm.needs_input();
                    const hasInput = vm.has_input();
                    const nextOpcode = vm.get_next_opcode ? vm.get_next_opcode() : 'N/A';
                    console.log('runBatch: needsInput=', needsInput, 'hasInput=', hasInput, 'nextOpcode=', nextOpcode);
                    
                    if (needsInput) {
                        updateStatus('Waiting for input', 'debug');
                        logToTerminal(`⌨️ Program requires ${needsInput === 'number' ? 'a number' : 'a character'} input`, 'prompt');
                        
                        showInputModal(needsInput, (value) => {
                            // Handle cancel
                            if (value === null) {
                                logToTerminal('⚠️ Input cancelled - stopping program', 'output');
                                setRunningEffect(false);
                                updateStatus('Stopped (input cancelled)', 'halted');
                                // Get output so far
                                const output = vm.ink_string();
                                if (output) {
                                    logToTerminal(`Output: ${output}`, 'program-output');
                                }
                                vm.reset();
                                highlightBytecodeRow(0);
                                return;
                            }
                            
                            console.log('Input received:', value, 'type:', typeof value);
                            // Provide input to VM
                            if (needsInput === 'number') {
                                vm.input(value);
                                logToTerminal(`📥 Input: ${value}`, 'output');
                            } else {
                                console.log('Calling vm.input_char with:', value);
                                vm.input_char(value);
                                logToTerminal(`📥 Input: '${String.fromCharCode(value)}'`, 'output');
                            }
                            
                            // Verify input was stored
                            console.log('After input_char: has_input=', vm.has_input());
                            
                            // Resume fast execution
                            updateStatus('Running (native)', 'running');
                            setTimeout(runBatch, 10);
                        });
                        return;
                    }
                    
                    // Execute a batch of steps (until halt, input needed, or max)
                    const batchSize = Math.min(10000, maxSteps - totalSteps);
                    console.log('Calling play with batchSize:', batchSize);
                    const stepsExecuted = vm.play(batchSize);
                    console.log('play returned:', stepsExecuted);
                    totalSteps += stepsExecuted;
                    
                    const snapshot = vm.snapshot();
                    
                    // Check if halted or max steps reached
                    if (snapshot.halted || totalSteps >= maxSteps) {
                        const endTime = performance.now();
                        const elapsed = Math.max(0.001, endTime - startTime);
                        
                        // Get output before reset
                        const output = vm.ink_string();
                        
                        if (output) {
                            logToTerminal(`Output: ${output}`, 'program-output');
                        }
                        
                        setRunningEffect(false);
                        updateStatus(`Completed in ${elapsed.toFixed(2)}ms`, 'halted');
                        logToTerminal(`✓ Executed ${totalSteps.toLocaleString()} steps in ${elapsed.toFixed(2)}ms`, 'output');
                        
                        // Show performance metrics
                        if (totalSteps > 0) {
                            const opsPerSec = (totalSteps / elapsed) * 1000;
                            if (opsPerSec >= 1000000) {
                                logToTerminal(`⚡ Speed: ${(opsPerSec / 1000000).toFixed(2)}M operations/second`, 'output');
                            } else {
                                logToTerminal(`⚡ Speed: ${(opsPerSec / 1000).toFixed(2)}K operations/second`, 'output');
                            }
                        }
                        
                        // Update UI with final state then reset for next run
                        updateExecutionStats(totalSteps, elapsed);
                        updateStackDisplay(snapshot.stack || []);
                        
                        // Reset VM for next run
                        vm.reset();
                        highlightBytecodeRow(0);
                        updatePositionDisplay(0, 0, 0);
                        updateDpCc('Right', 'Left');
                        logToTerminal('🔄 Ready for next run', 'output');
                        return;
                    }
                    
                    // Continue running (input might be needed)
                    setTimeout(runBatch, 0);
                };
                
                runBatch();
                
            } catch (error) {
                setRunningEffect(false);
                updateStatus('Error', 'halted');
                logToTerminal(`✗ Execution failed: ${error.message}`, 'error');
                console.error('Run error:', error);
            }
        }, 50); // Small delay to show effect
    });
}

if (playButton) {
    playButton.addEventListener('click', () => {
        if (!vm) {
            logToTerminal('✗ No program loaded. Upload a Piet image first.', 'error');
            return;
        }
        
        logToTerminal('▶ Executing program (animated)...', 'prompt');
        updateStatus('Running', 'running');
        setRunningEffect(true);
        
        try {
            // Reset VM to initial state
            try {
                vm.reset();
                highlightBytecodeRow(0);
            } catch (e) {
                logToTerminal('ℹ Reset not available, continuing from current state', 'prompt');
            }
            
            // Run until halt or error
            let step = 0;
            const maxSteps = 10000; // Safety limit
            const startTime = performance.now();
            
            const executeStep = () => {
                try {
                    const snapshot = vm.snapshot();
                    
                    // Highlight current instruction in bytecode panel
                    if (snapshot.instruction_index !== null && snapshot.instruction_index !== undefined) {
                        highlightBytecodeRow(snapshot.instruction_index);
                    }
                    
                    // Update displays
                    updateStackDisplay(snapshot.stack || []);
                    updatePositionDisplay(snapshot.position_x, snapshot.position_y, snapshot.instruction_index || 0);
                    updateDpCc(snapshot.direction, snapshot.codel_chooser);
                    updateExecutionStats(step, performance.now() - startTime);
                    
                    if (snapshot.halted || step >= maxSteps) {
                        const endTime = performance.now();
                        const output = vm.ink_string();
                        if (output) {
                            logToTerminal(`Output: ${output}`, 'program-output');
                        }
                        setRunningEffect(false);
                        updateStatus(`Completed in ${step} steps`, 'halted');
                        updateExecutionStats(step, endTime - startTime);
                        logToTerminal(`✓ Program completed in ${step} steps`, 'output');
                        
                        // Reset VM for next run
                        vm.reset();
                        highlightBytecodeRow(0);
                        updatePositionDisplay(0, 0, 0);
                        updateDpCc('Right', 'Left');
                        logToTerminal('🔄 Ready for next run', 'output');
                        return;
                    }
                    
                    // Check if input is needed
                    const needsInput = vm.needs_input();
                    if (step < 20 || needsInput) {
                        console.log(`Step ${step}: pos=(${snapshot.position_x},${snapshot.position_y}) needs_input=${needsInput} halted=${snapshot.halted}`);
                    }
                    if (needsInput) {
                        updateStatus('Waiting for input', 'debug');
                        logToTerminal(`⌨️ Program requires ${needsInput === 'number' ? 'a number' : 'a character'} input`, 'prompt');
                        
                        showInputModal(needsInput, (value) => {
                            // Handle cancel
                            if (value === null) {
                                logToTerminal('⚠️ Input cancelled - stopping program', 'output');
                                setRunningEffect(false);
                                updateStatus('Stopped (input cancelled)', 'halted');
                                const output = vm.ink_string();
                                if (output) {
                                    logToTerminal(`Output: ${output}`, 'program-output');
                                }
                                vm.reset();
                                highlightBytecodeRow(0);
                                return;
                            }
                            
                            // Provide input to VM
                            console.log('Callback received value:', value);
                            if (needsInput === 'number') {
                                vm.input(value);
                                logToTerminal(`📥 Input: ${value}`, 'output');
                            } else {
                                vm.input_char(value);
                                logToTerminal(`📥 Input: '${String.fromCharCode(value)}'`, 'output');
                            }
                            
                            // Verify input was received
                            console.log('After input - has_input:', vm.has_input());
                            console.log('After input - needs_input:', vm.needs_input());
                            
                            // Resume execution
                            updateStatus('Running', 'running');
                            setTimeout(executeStep, 50);
                        });
                        return;
                    }
                    
                    // Execute one step
                    vm.stroke();
                    
                    step++;
                    setTimeout(executeStep, 50); // Faster animation
                } catch (error) {
                    // Check if it's just halted
                    try {
                        const snapshot = vm.snapshot();
                        if (snapshot.halted) {
                            const endTime = performance.now();
                            const output = vm.ink_string();
                            if (output) {
                                logToTerminal(`Output: ${output}`, 'program-output');
                            }
                            setRunningEffect(false);
                            updateStatus(`Completed in ${step} steps`, 'halted');
                            updateExecutionStats(step, endTime - startTime);
                            logToTerminal(`✓ Program completed in ${step} steps`, 'output');
                            
                            // Reset VM for next run
                            vm.reset();
                            highlightBytecodeRow(0);
                            updatePositionDisplay(0, 0, 0);
                            updateDpCc('Right', 'Left');
                            logToTerminal('🔄 Ready for next run', 'output');
                            return;
                        }
                    } catch (e) {}
                    
                    setRunningEffect(false);
                    
                    // Check for watchdog timeout
                    const errorMsg = error.message || error.toString();
                    if (errorMsg.includes('timeout') || errorMsg.includes('Watchdog')) {
                        updateStatus('Timeout', 'halted');
                        logToTerminal(`⏱️ Watchdog timeout: Program exceeded step limit`, 'error');
                        logToTerminal(`💡 The program may have an infinite loop. Check for a proper HALT structure.`, 'output');
                    } else {
                        updateStatus('Error', 'halted');
                        logToTerminal(`✗ Runtime error: ${errorMsg}`, 'error');
                    }
                    console.error('Runtime error:', error);
                }
            };
            
            executeStep();
        } catch (error) {
            setRunningEffect(false);
            updateStatus('Error', 'halted');
            logToTerminal(`✗ Execution failed: ${error.message}`, 'error');
            console.error('Execution error:', error);
        }
    });
}

if (stepButton) {
    stepButton.addEventListener('click', () => {
        if (!vm || !program) {
            logToTerminal('✗ No program loaded. Upload a Piet image first.', 'error');
            return;
        }
        
        updateStatus('Stepping', 'debug');
        
        try {
            const snapshot = vm.snapshot();
            
            // Start debug visualization on first step (if not already in debug mode)
            if (!debugMode && snapshot.steps === 0) {
                startDebugVisualization(snapshot.codel_size);
                logToTerminal('🔍 Debug mode: grayscale view, executed codels will be colored', 'prompt');
            }
            
            if (snapshot.halted) {
                logToTerminal('✓ Program completed (HALT). Reset or upload new image to restart.', 'output');
                highlightBytecodeRow(snapshot.instruction_index);
                endDebugVisualization();
                updateStatus('Halted', 'halted');
                return;
            }
            
            // Highlight current codel on canvas (uses debug visualization if active)
            console.log('Step - calling highlightCodel with:', snapshot.pixel_x, snapshot.pixel_y, snapshot.codel_size);
            console.log('originalImageData is:', originalImageData ? `${originalImageData.width}x${originalImageData.height}` : 'NULL');
            highlightCodel(snapshot.pixel_x, snapshot.pixel_y, snapshot.codel_size);
            
            // Highlight current instruction BEFORE execution
            if (snapshot.instruction_index !== null && snapshot.instruction_index !== undefined) {
                highlightBytecodeRow(snapshot.instruction_index);
            }
            
            // Update displays
            updateStackDisplay(snapshot.stack || []);
            updatePositionDisplay(snapshot.position_x, snapshot.position_y, snapshot.instruction_index || 0);
            updateDpCc(snapshot.direction, snapshot.codel_chooser);
            
            // Check if input is needed before executing
            const needsInput = vm.needs_input();
            if (needsInput) {
                updateStatus('Waiting for input', 'debug');
                logToTerminal(`⌨️ Program requires ${needsInput === 'number' ? 'a number' : 'a character'} input`, 'prompt');
                
                showInputModal(needsInput, (value) => {
                    // Handle cancel
                    if (value === null) {
                        logToTerminal('⚠️ Input cancelled. Click Step to try again or Reset.', 'output');
                        updateStatus('Input cancelled', 'debug');
                        return;
                    }
                    
                    // Provide input to VM
                    if (needsInput === 'number') {
                        vm.input(value);
                        logToTerminal(`📥 Input: ${value}`, 'output');
                    } else {
                        vm.input_char(value);
                        logToTerminal(`📥 Input: '${String.fromCharCode(value)}'`, 'output');
                    }
                    
                    updateStatus('Stepping', 'debug');
                    logToTerminal('Input provided. Click Step again to continue.', 'output');
                });
                return;
            }
            
            // Execute one step
            vm.stroke();
            
            // Get updated snapshot
            const afterSnapshot = vm.snapshot();
            
            // Update highlight to new position
            highlightCodel(afterSnapshot.pixel_x, afterSnapshot.pixel_y, afterSnapshot.codel_size);
            
            // Update displays after execution
            if (afterSnapshot.instruction_index !== null && afterSnapshot.instruction_index !== undefined) {
                highlightBytecodeRow(afterSnapshot.instruction_index);
            }
            updateStackDisplay(afterSnapshot.stack || []);
            updatePositionDisplay(afterSnapshot.position_x, afterSnapshot.position_y, afterSnapshot.instruction_index || 0);
            updateDpCc(afterSnapshot.direction, afterSnapshot.codel_chooser);
            updateExecutionStats(afterSnapshot.steps, 0);
            
            // Show any new output
            const output = vm.ink_string();
            if (output) {
                logToTerminal(`Output: ${output}`, 'program-output');
            }
            
            logToTerminal(`Step ${snapshot.steps}: instruction #${snapshot.instruction_index}`, 'prompt');
            
            if (afterSnapshot.halted) {
                endDebugVisualization();
                updateStatus('Halted', 'halted');
                logToTerminal('✓ Program completed (HALT)', 'output');
            }
        } catch (error) {
            // Check if it's just a halted state
            try {
                const snapshot = vm.snapshot();
                if (snapshot.halted) {
                    endDebugVisualization();
                    updateStatus('Halted', 'halted');
                    logToTerminal('✓ Program completed (HALT)', 'output');
                    return;
                }
            } catch (e) {}
            
            endDebugVisualization();
            updateStatus('Error', 'halted');
            logToTerminal(`✗ Step failed: ${error.message}`, 'error');
            console.error('Step error:', error);
        }
    });
}

// Initial messages
logToTerminal('Canvas VM Ready', 'output');
logToTerminal('Upload a Piet program (PNG/BMP) to begin', 'output');

}); // End DOMContentLoaded
