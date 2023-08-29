window.onload = function() {
    document.body.addEventListener('htmx:responseError', function(evt) {
    console.log(evt.detail);
    });
};
