document.body.addEventListener('htmx:responseError', function(evt) {
  console.log(evt.detail);
  let detail = evt.detail;
  let responsecode = detail.xhr.status;
  if (responsecode == 400 && detail.requestConfig.path === "/list/") {
    alert(detail.xhr.response)
    console.log(evt.detail.xhr.repsonse);
  }
});

// document.body.addEventListener('htmx:beforeRequest', function(evt) {
//   let detail = evt.detail;
//   console.log(evt.detail);
//   return false;
// });
console.log("Added event listener");
