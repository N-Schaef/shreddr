var tagMap = new Map()
function filterByTag(tagId) {
  var tags = JSON.parse(sessionStorage.getItem("filterTags")) || [];
  tags.push(tagId)
  sessionStorage.setItem("filterTags", JSON.stringify(Array.from(new Set(tags))));
  location.reload();
}

function removeTagFilter(tagId) {
  var tags = JSON.parse(sessionStorage.getItem("filterTags")) || [];
  const index = tags.indexOf(tagId);
  if (index > -1) {
    tags.splice(index, 1);
  }
  sessionStorage.setItem("filterTags", JSON.stringify(Array.from(new Set(tags))));
  location.reload();
}



function createSearchTagButton(){
  var div = $("<div></div>");
  var dropdownButton = $("<button class=\"btn btn-sm btn-outline-secondary dropdown-toggle mr-1\" type=\"button\" id=\"addTagButton\" data-toggle=\"dropdown\"><span data-feather=\"tag\"></span></button>");
  var items = $("<div class=\"dropdown-menu\" aria-labelledby=\"addTagButton\"></div>");
  var filter = $("<input class=\"form-control\" id=\"filterTags\" type=\"text\" placeholder=\"Search..\">");
  filter.on("keyup", function() {
    var value = $(this).val().toLowerCase();
    $(".dropdown-menu button").filter(function() {
      $(this).toggle($(this).text().toLowerCase().indexOf(value) > -1)
    });
  });

  items.append(filter);
  tagMap.forEach(function (value, key){
    var item = $("<button class=\"dropdown-item\" type=\"button\"></button>");
    item.text(value.name);
    item.css("background-color", value.color);
    item.on("click", function(){
      filterByTag(key)
    }.bind(key));
    items.append(item);
  });
  div.append(dropdownButton);
  div.append(items);
  return div;
}

function createTagButton(tagId, clickFunc) {
  var btn = $("<button type=\"button\" class=\"btn btn-sm mb-1 tag-btn\">unknown tag</button>");
  var tag = tagMap.get(tagId);
  if (!tag) {
    return "";
  }
  btn.text(tag.name);
  btn.css("background-color", tag.color);
  btn.addClass("tag-" + tag.id);
  btn.on("click", clickFunc);
  return btn;
}

function activateDoc(event) {
  event.target.removeClass("shadow-sm");
  event.target.addClass("shadow-lg");
}

function deactivateDoc(event) {
  event.target.removeClass("shadow-lg");
  event.target.addClass("shadow-sm");
}

function nextHandler(pageIndex){
  let searchParams = new URLSearchParams(window.location.search)
  var paramObj = {};
  let limit = 10;
  paramObj.count = limit;
  paramObj.offset = pageIndex*limit;
  if (searchParams.has('order')) {
    paramObj.order = searchParams.get('order')
  }
  if (searchParams.has('query')) {
    paramObj.query = searchParams.get('query')
  }
  let tags =[];
  let tmp = sessionStorage.getItem("filterTags");
  if(tmp && tmp != null){
    tags = JSON.parse(tmp);
  }

  paramObj.tag = tags.join();

  return fetch("/api/documents?"+$.param(paramObj))
  .then(response => response.json())
  .then((data) => {
    let frag = document.createDocumentFragment();
    data.forEach(function (docData) {
      let item = createDocumentCard(docData)
      frag.appendChild(item[ 0 ]);
    });
    let hasNextPage = data.length == limit;
    return this.append(Array.from(frag.childNodes))
    .then(() => hasNextPage);
  });
}

function createDocumentCard(doc) {
  const template = `
  <div class="card shadow-sm h-100" id="doc-${doc.id}">
        <div class="card-header">
        <a href="/documents/${doc.id}" class="text-dark" style="text-decoration: none;"><h5 class="card-title">${doc.title}</h5></a>
        </div>
        <img class="card-img-bottom" src="/thumbnails/${doc.id}.jpg" rel="nofollow" alt="Document thumbnail">
        <div class="card-body">
          <p class="card-text"></p>
        </div>

        <div class="btn-group docbuttons" style="display: none;">
          <a href="/documents/${doc.id}/download" title="Download document" class="btn btn-primary download-doc"
            style="border-radius: 0 !important;">${feather.icons['download'].toSvg()}</a>
          <button type="button" title="Reprocess document" class="btn btn-secondary reimport-doc"
            style="border-radius: 0 !important;">${feather.icons['refresh-cw'].toSvg()}</button>
          <a href="/documents/${doc.id}" title="Edit document metadata" class="btn btn-secondary edit-doc"
            style="border-radius: 0 !important;">${feather.icons['edit'].toSvg()}</a>
          <button type="button" title="Remove document" class="btn btn-danger remove-doc"
            style="border-radius: 0 !important;">${feather.icons['trash-2'].toSvg()}</button>
        </div>

        <div class="card-footer text-muted">
          <div class="created">Imported Yesterday</div>
          <div class="inferred"></div>
        </div>

      </div>
  `

  
  let card = $(template.trim());
  card.mouseover(function () {
    $(this).removeClass("shadow-sm");
    $(this).addClass("shadow-lg");
    $(this).find(".docbuttons").show();
  });
  card.mouseout(function () {
    $(this).removeClass("shadow-lg");
    $(this).addClass("shadow-sm");
    $(this).find(".docbuttons").hide();
  });


  card.find(".reimport-doc").on('click', function () {
    $.ajax({
      url: "/api/documents/" + doc.id + "/reimport",
      type: 'GET',
      success: function (result) {
        updateStatusWindow();
      }
    });
  }.bind(doc)
  );

  card.find(".remove-doc").on('click', function () {
    $.ajax({
      url: "/api/documents/" + doc.id,
      type: 'DELETE',
      success: function (result) {
        location.reload();
      }
    });
  }.bind(doc)
  );

  var tags = card.find(".card-text");
  doc.tags.forEach(function (tag) {
    tags.append(createTagButton(tag, function () { filterByTag(tag) }.bind(tag)));
  });
  var date = new Date(0);
  date.setUTCSeconds(doc.imported_date)
  card.find(".created").text("Imported: " + date.toLocaleDateString() + " - " + date.toLocaleTimeString());
  if (doc.extracted.doc_date) {
    date = new Date(0);
    date.setUTCSeconds(doc.extracted.doc_date);
    card.find(".inferred").text("Document: " + date.toLocaleDateString());
  }
  let cardDiv = $("<div class=\"col-11 col-sm-10 col-md-6 col-lg-4 col-xl-2 py-2 \"></div>")
  cardDiv.html(card);
  return cardDiv;
}

(function () {
  'use strict'

  let searchParams = new URLSearchParams(window.location.search)
  var order = 0;
  if (searchParams.has('order')) {
    order = searchParams.get('order')
  }
  var page = 1;
  if (searchParams.has('page')) {
    page = searchParams.get('page')
  }

  var tags = JSON.parse(sessionStorage.getItem("filterTags")) || [];
  tagMap = new Map()
  $.get("/api/tags")
    .done(function (data) {
      data.forEach(function (tag) {
        tagMap.set(tag.id, tag);
      });

      //Show filters
      var filters = $("#filter-content")
      tags.forEach(function (tag) {
        filters.append(createTagButton(tag, function () { removeTagFilter(tag) }.bind(tag)));
      });

      $("#filter-btn").html(createSearchTagButton());
      feather.replace();

      bsCustomFileInput.init()

     window.ias = new InfiniteAjaxScroll('#documents',{
       item: '.card',
       next: nextHandler,
     })

    });


    $('#uploadButton').on('click', function () {
      $.ajax({
        // Your server script to process the upload
        url: '/api/documents',
        type: 'POST',
    
        // Form data
        data: new FormData($('#uploadForm')[0]),
    
        // Tell jQuery not to process data or worry about content-type
        // You *must* include these options!
        cache: false,
        contentType: false,
        processData: false,
    
        // Custom XMLHttpRequest
        xhr: function () {
          var myXhr = $.ajaxSettings.xhr();
          if (myXhr.upload) {
            $('#status').text("Uploading...");
            updateStatusWindow();
            // For handling the progress of the upload
          }
          return myXhr;
        }
      }).done(function(){
        $('#status').text("Uploaded!");
      }).fail(function(){
        $('#status').text("Could not upload!");
      });
    });


})()

function updateStatusWindow(){
  $.get("/api/job").done(function (data) {
    let alert = $("#job_alert");
    if (data == "Idle"){
      if(alert.is(":visible")){
        alert.removeClass("alert-primary");
        alert.addClass("alert-success");
        alert.find("#alertText").text("Finished jobs. Refresh for new content.");
      }
    }else{
        var text = "Processing " + (data.Busy.queue+1) + " documents. "
        text += data.Busy.current;
        alert.find("#alertText").text(text);
        alert.removeClass("alert-success");
        alert.addClass("alert-primary");
        alert.show();
    }
  });
}

window.setInterval(function(){
  updateStatusWindow();
}, 5000);