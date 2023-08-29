window.onload = function() {
    document.body.addEventListener('htmx:responseError', function(evt) {
        console.log(evt.detail);
    });

    document.dispatchEvent(new Event("loaded"));
};

function is_positive_integer(val) {
    return /^\d+$/.test(val);
}
