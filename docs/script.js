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
let program = null;
let currentImage = null;

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
        
        // Load image into VM
        const width = imageData.width;
        const height = imageData.height;
        const rgbaData = new Uint8Array(imageData.data);
        
        vm.paint(rgbaData, width, height);
        
        logToTerminal(`✓ Program loaded: ${width}x${height} pixels`, 'output');
        
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
                
                // Show canvas, hide upload area
                canvas.style.display = 'block';
                if (uploadArea) uploadArea.style.display = 'none';
                
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

if (playButton) {
    playButton.addEventListener('click', () => {
        if (!vm) {
            logToTerminal('✗ No program loaded. Upload a Piet image first.', 'error');
            return;
        }
        
        logToTerminal('▶ Executing program...', 'prompt');
        
        try {
            // Reset VM to initial state
            try {
                vm.reset();
                highlightBytecodeRow(0); // Highlight first instruction
            } catch (e) {
                logToTerminal('ℹ Reset not available, continuing from current state', 'prompt');
            }
            
            // Run until halt or error
            let step = 0;
            const maxSteps = 10000; // Safety limit
            
            const executeStep = () => {
                try {
                    const snapshot = vm.snapshot();
                    
                    // Highlight current instruction
                    if (snapshot.instruction_index !== null && snapshot.instruction_index !== undefined) {
                        highlightBytecodeRow(snapshot.instruction_index);
                    }
                    
                    // Update stack display
                    updateStackDisplay(snapshot.stack || []);
                    
                    if (snapshot.halted || step >= maxSteps) {
                        const output = vm.ink_string();
                        console.log('ink_string returned:', output, 'type:', typeof output);
                        logToTerminal(`Output: ${output || '(empty)'}`, 'program-output');
                        logToTerminal(`✓ Program completed in ${step} steps`, 'output');
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
                            const output = vm.ink_string();
                            console.log('ink_string (error path):', output);
                            logToTerminal(`Output: ${output || '(empty)'}`, 'program-output');
                            logToTerminal(`✓ Program completed in ${step} steps`, 'output');
                            return;
                        }
                    } catch (e) {}
                    
                    logToTerminal(`✗ Runtime error: ${error.message}`, 'error');
                    console.error('Runtime error:', error);
                }
            };
            
            executeStep();
        } catch (error) {
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
        
        try {
            const snapshot = vm.snapshot();
            
            if (snapshot.halted) {
                logToTerminal('✓ Program completed (HALT). Reset or upload new image to restart.', 'output');
                highlightBytecodeRow(snapshot.instruction_index);
                return;
            }
            
            // Highlight current instruction BEFORE execution
            if (snapshot.instruction_index !== null && snapshot.instruction_index !== undefined) {
                highlightBytecodeRow(snapshot.instruction_index);
            }
            
            // Update stack display
            updateStackDisplay(snapshot.stack || []);
            
            // Execute one step
            vm.stroke();
            
            // Get updated snapshot
            const afterSnapshot = vm.snapshot();
            
            // Update highlight after execution
            if (afterSnapshot.instruction_index !== null && afterSnapshot.instruction_index !== undefined) {
                highlightBytecodeRow(afterSnapshot.instruction_index);
            }
            
            // Update stack after execution
            updateStackDisplay(afterSnapshot.stack || []);
            
            // Show any new output
            const output = vm.ink_string();
            if (output) {
                logToTerminal(`Output: ${output}`, 'program-output');
            }
            
            logToTerminal(`Step ${snapshot.steps}: executed instruction #${snapshot.instruction_index}`, 'prompt');
            
            if (afterSnapshot.halted) {
                logToTerminal('✓ Program completed (HALT)', 'output');
            }
        } catch (error) {
            // Check if it's just a halted state (not a real error)
            try {
                const snapshot = vm.snapshot();
                if (snapshot.halted) {
                    logToTerminal('✓ Program completed (HALT)', 'output');
                    return;
                }
            } catch (e) {}
            
            logToTerminal(`✗ Step failed: ${error.message}`, 'error');
            console.error('Step error:', error);
        }
    });
}

// Initial messages
logToTerminal('Canvas VM Ready', 'output');
logToTerminal('Upload a Piet program (PNG/BMP) to begin', 'output');

}); // End DOMContentLoaded
