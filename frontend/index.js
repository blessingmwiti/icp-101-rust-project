function searchProduct() {
    // Get the product ID from the input
    const productId = document.getElementById('productId').value;

    // Fetch product details from the Canister using the product ID
    fetch(`http://127.0.0.1:4943/canister-api-endpoint/get_product`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({
            id: productId,
        }),
    })
        .then(response => response.json())
        .then(data => {
            // Display product details in the table
            displayProductDetails(data);
        })
        .catch(error => {
            console.error(error);
            // Show an alert if there is an error or no product found
            alert('Error fetching product or product not found.');
        });
}

function displayProductDetails(product) {
    const productTable = document.getElementById('productTable');
    const tbody = productTable.getElementsByTagName('tbody')[0];
    tbody.innerHTML = ''; // Clear previous data

    if (product) {
        // Create a new row for the product
        const row = tbody.insertRow();
        const cellId = row.insertCell(0);
        const cellName = row.insertCell(1);
        const cellOrigin = row.insertCell(2);

        // Populate cells with product details
        cellId.innerHTML = product.id;
        cellName.innerHTML = product.name;
        cellOrigin.innerHTML = product.origin;
        // Add more cells as needed for additional product details

        // Show the table
        productTable.style.display = 'table';
    } else {
        // Hide the table and show an alert if no product is found
        productTable.style.display = 'none';
        alert('No product found with the given ID.');
    }
}
