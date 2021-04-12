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
  var items = $("<div class=\"tag-filter-menu dropdown-menu\" aria-labelledby=\"addTagButton\"></div>");
  var filter = $("<input class=\"form-control\" id=\"filterTags\" type=\"text\" placeholder=\"Search...\">");

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
    item.css("color", isDark(value.color) ? "var(--light)" : "var(--dark)")
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
  var btn = $("<button type=\"button\" class=\"btn btn-sm tag-btn\"><div class=\"tag-label\">unknown tag</div></button>");
  var tag = tagMap.get(tagId);
  if (!tag) {
    return "";
  }
  btn.find(".tag-label").text(tag.name);
  btn.css("background-color", tag.color);
  btn.css("color", isDark(tag.color) ? "var(--light)" : "var(--dark)")
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

  return fetch("/documents/json?"+$.param(paramObj))
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
  <div class="doc-card card shadow-sm h-100" id="doc-${doc.id}" data-doc_id="${doc.id}">
    <div class="card-header" style="padding: .5rem;">
      <a href="/documents/${doc.id}" class="text-dark" style="text-decoration: none;">
        <div class="doc-title">${doc.title}</div>
      </a>
    </div>

    <div class="docbutton-container">
      <div class="btn-group docbuttons w-100" style="display: none;">
        <a href="/documents/${doc.id}/download" title="Download document" class="btn btn-primary w-100 download-doc"
          style="border-radius: 0 !important;">${feather.icons['download'].toSvg()}</a>
        <button type="button" title="Reprocess document" class="btn btn-secondary w-100 reimport-doc"
          style="border-radius: 0 !important;">${feather.icons['refresh-cw'].toSvg()}</button>
        <a href="/documents/${doc.id}" title="Edit document metadata" class="btn btn-secondary w-100 edit-doc"
          style="border-radius: 0 !important;">${feather.icons['edit'].toSvg()}</a>
        <button type="button" title="Remove document" class="btn btn-danger w-100 remove-doc"
          style="border-radius: 0 !important;">${feather.icons['trash-2'].toSvg()}</button>
      </div>
    </div>

    <img class="card-img-bottom doc-image" src="/thumbnails/${doc.id}.jpg" rel="nofollow" alt="Document thumbnail">
    <div class="card-body">
    <p class="card-text"></p>
    </div>

    <div class="card-footer text-muted" style="padding: .5rem;">
      <div class="created">Imported Yesterday</div>
      <div class="inferred"></div>
    </div>

  </div>
  `

  
  let card = $(template.trim());
  card.mouseover(function () {
    $(this).removeClass("shadow-sm");
    $(this).addClass("shadow-lg");
    if (!$("#documents").hasClass("doc-multiselect")) {
      // Do not show buttons in selection mode
      $(this).find(".docbuttons").show();
    }
  });
  card.mouseout(function () {
    $(this).removeClass("shadow-lg");
    $(this).addClass("shadow-sm");
    $(this).find(".docbuttons").hide();
  });


  card.find(".reimport-doc").on('click', function () {
    $.ajax({
      url: "/documents/" + doc.id + "/reimport",
      type: 'PUT',
      success: function (result) {
        updateStatusWindow();
      }
    });
  }.bind(doc)
  );

  card.find(".remove-doc").on('click', function () {
    $.ajax({
      url: "/documents/" + doc.id,
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
  let cardDiv = $("<div class=\"col-12 col-sm-6 col-md-6 col-lg-4 col-xl-2 py-2 \"></div>")
  cardDiv.html(card);
  return cardDiv;
}

function toggleMultiSelect(elem) {
  var docbuttons = $("#multiselect-docbuttons");

  if (!elem.classList.contains("active")) {
    // If button is currently inactive, we are on our way
    // to multi-select mode
    $("#documents").addClass("doc-multiselect");
    docbuttons.addClass("d-flex");
    docbuttons.removeClass("d-none");
    $(".doc-card").on("click", function() {
      $(this).toggleClass("selected");
    })

    docbuttons.find(".tag-filter").on("keyup", function () {
      var value = $(this).val().toLowerCase();
      docbuttons.find(".add-tag-menu .btn-group").filter(function () {
        let contains = $(this).find(".btn-add-tag").text().toLowerCase().indexOf(value) > -1;
        $(this).toggle(contains);
      });
    });

    // Remove old set of tags
    docbuttons.find(".tag-container").empty();

    // Fill with fresh tags from server
    $.get("/tags/json").done(function (data) {
      data.forEach(function (tag) {
        var item = $("<div class=\"btn-group\"><button class=\"btn btn-sm btn-add-tag\" type=\"button\"></button><button class=\"btn btn-sm btn-danger btn-del-tag\" type=\"button\">X</button></div>");
        var add_button = item.find(".btn-add-tag")
        var del_button = item.find(".btn-del-tag");

        add_button.text(tag.name);
        add_button.css("background-color", tag.color);
        add_button.css("color", isDark(tag.color) ? "var(--light)" : "var(--dark)");
        add_button.on("click", () => addTagToSelected(tag));

        del_button.on("click", () => removeTagFromSelected(tag))

        docbuttons.find(".tag-container").append(item);
      });
    });

  } else {
    // Button has .active, so user wants to toggle multi-select
    // mode off
    $("#documents").removeClass("doc-multiselect")
    docbuttons.addClass("d-none");
    docbuttons.removeClass("d-flex");
    docbuttons.find(".tag-filter").unbind("keyup");
    $(".doc-card").removeClass("selected");
    $(".doc-card").unbind("click")
  }
}

function reprocessSelected() {
  $(".doc-card.selected").each(function() {
    $.ajax({
      url: "/documents/" + this.dataset.doc_id + "/reimport",
      type: 'PUT',
      success: function (result) {
        updateStatusWindow();
      }
    });
  });
}

function removeSelected() {
  $(".doc-card.selected").each(function() {
    $.ajax({
      url: "/documents/" + this.dataset.doc_id,
      type: 'DELETE',
      success: () => {
        console.log(`DELETE for doc ${this.dataset.doc_id}`);
      }
    });
  });
}

function addTagToSelected(tag) {
  $(".doc-card.selected").each(function() {
    $.ajax({
      url: '/documents/' + this.dataset.doc_id + '/tags/' + tag.id,
      type: 'POST',
      success: () => {
        console.log(`POST for doc ${this.dataset.doc_id}, tag ${tag.id}`);
      }
    });
  });
}

function removeTagFromSelected(tag) {
  $(".doc-card.selected").each(function() {
    $.ajax({
      url: '/documents/' + this.dataset.doc_id + '/tags/' + tag.id,
      type: 'DELETE',
      success: () => {
        console.log(`POST for doc ${this.dataset.doc_id}, tag ${tag.id}`);
      }
    });
  });
}

function selectAll() {
  $(".doc-card").addClass("selected");
}

function deselectAll() {
  $(".doc-card").removeClass("selected");
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
  $.get("/tags/json")
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

     window.ias = new InfiniteAjaxScroll('#documents',{
       item: '.card',
       next: nextHandler,
     })

    });


    $('#uploadButton').on('click', function () {
      var fileList = $('#customFile').prop("files");
      for (var i = 0; i < fileList.length; i++) {
        var form_data = new FormData();
        form_data.append("file", fileList[i]);
        $.ajax({
          // Your server script to process the upload
          url: '/documents',
          type: 'POST',
      
          // Form data
          data: form_data,
      
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
      }


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