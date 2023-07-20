import * as wasm from "wasm-farm";

// wasm.greet();
console.log("demo")

const store_value_button = document.getElementById('store_value')

const vegetable_text = document.getElementById('vegetable')
const amount_text = document.getElementById('amount')
const weight_text = document.getElementById('weight')
const selected_date = document.getElementById('date')
const return_text = document.getElementById('return')

var myTimer = setInterval(update_table_data, 10);
var update_table = true

const text_handler = async function(e) {
    console.log("handle text")
    await wasm.add_to_db(vegetable_text.value, amount_text.value, weight_text.value, selected_date.value)
    update_table = true
    clearInterval(myTimer);
    myTimer = setInterval(update_table_data, 1000);
}

async function update_table_data() {
    if (update_table) {
        updateTableWithWasmData()
        update_table = false
    }
}

store_value_button.addEventListener('click', text_handler)

// setInterval(updateData, 500);
async function updateData() {
    let query_val =  await wasm.get_db_data()
    return_text.value = query_val
    console.log(query_val)
}

const addRowToTable = (table, rowData) => {
    const newRow = table.insertRow();
    const { vegetable, number_of_veg, weight, date_picked } = rowData;
  
    const cell1 = newRow.insertCell(0);
    cell1.textContent = vegetable;
  
    const cell2 = newRow.insertCell(1);
    cell2.textContent = number_of_veg;
  
    const cell3 = newRow.insertCell(2);
    cell3.textContent = weight;
  
    const cell4 = newRow.insertCell(3);
    cell4.textContent = date_picked;
};

// script.js
// setInterval(updateTableWithWasmData, 10000);
async function updateTableWithWasmData() {
    const outputTableBody = document.getElementById('data-table-body');
    outputTableBody.innerHTML = ''; // Clear previous content
  
    // Call the Rust Wasm function to get the data
    const { get_list_of_lists } = module;
    const list_of_lists = await wasm.get_db_data();

    list_of_lists.forEach((row) => {
    const newRow = document.createElement('tr');
    row.forEach((item) => {
        const newCell = document.createElement('td');
        
        newCell.textContent = item;
        
        newRow.appendChild(newCell);
    });
    outputTableBody.appendChild(newRow);
    });
    clearInterval(myTimer);
}
  