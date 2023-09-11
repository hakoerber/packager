window.onload = function() {
    document.body.addEventListener('htmx:responseError', function(evt) {
        console.log(evt.detail);
    });

    document.dispatchEvent(new Event("loaded"));
};

function is_positive_integer(val) {
    return /^\d+$/.test(val);
}

function inventory_new_item_check_input() {
    return document.getElementById('new-item-name').value.length != 0
    && is_positive_integer(document.getElementById('new-item-weight').value)
}
function check_weight() {
    return document.getElementById('new-item-weight').validity.valid;
}
