function displayDate(ts) {
  if (ts == 0) return "-";
  var date = new Date(0);
  date.setUTCSeconds(ts);
  return date.toLocaleDateString();
}

function displayDateTime(ts) {
  if (ts == 0) return "-";
  var date = new Date(0);
  date.setUTCSeconds(ts);
  return date.toLocaleDateString() + " - " + date.toLocaleTimeString();
}

function updateTitle() {
  var patch = { "title": $("#doctitle").text() };
  $.ajax({
    type: 'PATCH',
    contentType: 'application/json',
    data: JSON.stringify(patch),
  });
}

function updateLanguage() {
  var patch = { "language": $("#lang").text() };
  $.ajax({
    type: 'PATCH',
    contentType: 'application/json',
    data: JSON.stringify(patch),
  });
}


function updateDate() {
  var patch = { "extracted": { "doc_date": parseInt($("#docDateInput").val()) } };
  $.ajax({
    type: 'PATCH',
    contentType: 'application/json',
    data: JSON.stringify(patch),
  });
}

function removeExtracted(extracted, type, index) {
  var newMeta = extracted;
  newMeta.splice(index, 1);
  var patch = { "extracted": { "doc_date": null } };
  patch.extracted[type] = newMeta;
  $.ajax({
    type: 'PATCH',
    contentType: 'application/json',
    data: JSON.stringify(patch),
    success: function (result) {
      location.reload();
    },
  });
}

function initButtons(docId) {
  //Reimport
  $('#reimport').on('click', function () {
    $.ajax({
      url: "/documents/" + docId + "/reimport",
      type: 'PUT'
    });
  }.bind(docId));
  //Reimport w OCR
  $('#force-ocr').on('click', function () {
    $.ajax({
      url: "/documents/" + docId + "/reimport?ocr=true",
      type: 'PUT'
    });
  }.bind(docId));


  //Title change
  $("#doctitle").on("keypress", function (e) { if (e.which == 13) { $("#doctitle").blur(); return false; } });
  $("#doctitle").on("focusout", function (e) { updateTitle(); });
  //Language change
  $("#lang").on("keypress", function (e) { if (e.which == 13) { $("#lang").blur(); return false; } });
  $("#lang").on("focusout", function (e) { updateLanguage(); });

}

function createTagButton(docId, tagMap, tagId, clickFunc) {
  var tag = tagMap.get(tagId);
  if (!tag) {
    return "";
  }
  var btn = $("<a href=\"/tags/" + tagId + "\" type=\"button\" class=\"btn btn-secondary btn-sm tag-btn mb-2\" style=\"border-color: transparent;\"></a>");
  btn.text(tag.name);
  btn.css("background-color", tag.color);
  btn.css("color", isDark(tag.color) ? "var(--light)" : "var(--dark)")
  btn.addClass("tag-" + tag.id);
  btn.on("click", clickFunc);
  var removebtn = $("<button type=\"button\" class=\"btn btn-sm btn-secondary tag-btn mb-2\" style=\"border-color: transparent;\"><span data-feather=\"x\"></span></button>");
  removebtn.css("background-color", tag.color);
  removebtn.css("color", isDark(tag.color) ? "var(--light)" : "var(--dark)")
  removebtn.on("click", function () {
    $.ajax({
      url: '/documents/' + docId + '/tags/' + tag.id,
      type: 'DELETE',
      success: function (result) {
        location.reload();
      }
    });
  }.bind(tag));
  var btnGroup = $("<div class=\"btn-group mr-2\"></div>")
  btnGroup.append(btn);
  btnGroup.append(removebtn);
  return btnGroup;
}

function createAddButton(docId, tagMap) {
  var div = $("<div></div>");
  var dropdownButton = $("<button class=\"btn btn-sm btn-success dropdown-toggle\" type=\"button\" id=\"addTagButton\" data-toggle=\"dropdown\"><span data-feather=\"plus\"></span></button>");
  var items = $("<div class=\"dropdown-menu tag-add-menu\" aria-labelledby=\"addTagButton\"></div>");
  var filter = $("<input class=\"form-control\" id=\"filterTags\" type=\"text\" placeholder=\"Search..\">");
  filter.on("keyup", function () {
    var value = $(this).val().toLowerCase();
    $(".dropdown-menu button").filter(function () {
      $(this).toggle($(this).text().toLowerCase().indexOf(value) > -1)
    });
  });

  items.append(filter);
  tagMap.forEach(function (value, key) {
    var item = $("<button class=\"dropdown-item\" type=\"button\"></button>");
    item.text(value.name);
    item.css("background-color", value.color);
    item.css("color", isDark(value.color) ? "var(--light)" : "var(--dark)");
    item.on("click", function () {
      $.ajax({
        url: '/documents/' + docId + '/tags/' + value.id,
        type: 'POST',
        success: function (result) {
          location.reload();
        }
      });
    }.bind(docId, value));
    items.append(item);
  });
  div.append(dropdownButton);
  div.append(items);
  return div;
}

function createTagButtons(docId, tags) {
  $.get("/tags/json")
    .done(function (data) {
      let tagMap = new Map()
      data.forEach(function (tag) {
        tagMap.set(tag.id, tag);
      });
      tags.forEach(function (tag) {
        $("#tags").append(createTagButton(docId, tagMap, tag, function () { }));
        tagMap.delete(tag);
      });
      $("#tags").append(createAddButton(docId, tagMap));

      feather.replace()
    });
}


$('#datepicker').datepicker();
$('#datepicker').on('changeDate', function () {
  let val = $('#datepicker').datepicker('getUTCDate').getTime() / 1000;
  $('#docDateInput').val(val);
  $('#docDate').text(displayDate(val))
  updateDate();
});



function initExtracted(extracted) {
  if (extracted.link.length > 0) {
    var i = 0;
    extracted.link.forEach(function (link) {
      if (!link.startsWith("http")) {
        link = "http://" + link;
      }
      const row = `<tr><td><a href="${link}">${link}</a><i><span class="text-muted" data-feather=\"x\"></span></i></td></tr>`;
      let elt = $(row)
      elt.find("i").on("click", removeExtracted.bind(null, extracted.link, "link", i));
      i++;
      $('#urlTable > tbody:last-child').append(elt);
    });
    $('#metaHeader').show();
    $('#URLS-div').show();
  }

  if (extracted.email.length > 0) {
    var i = 0;
    extracted.email.forEach(function (email) {
      const row = `<tr><td><a href="mailto:${email}">${email}</a><i><span class="text-muted" data-feather=\"x\"></span></i></td></tr>`;
      let elt = $(row)
      elt.find("i").on("click", removeExtracted.bind(null, extracted.email, "email", i));
      i++;
      $('#emailsTable > tbody:last-child').append(elt);
    });
    $('#metaHeader').show();
    $('#emails-div').show();
  }

  if (extracted.phone.length > 0) {
    var i = 0;
    extracted.phone.forEach(function (phone) {
      const row = `<tr><td><a href="tel:${phone}">${phone}</a><i><span class="text-muted" data-feather=\"x\"></span></i></td></tr>`;
      let elt = $(row)
      elt.find("i").on("click", removeExtracted.bind(null, extracted.phone, "phone", i));
      i++;
      $('#phoneTable > tbody:last-child').append(elt);
    });
    $('#metaHeader').show();
    $('#Phones-div').show();
  }

  if (extracted.iban.length > 0) {
    var i = 0;
    extracted.iban.forEach(function (iban) {
      const row = `<tr><td>${iban}<i><span class="text-muted" data-feather=\"x\"></span></i></td></tr>`;
      let elt = $(row)
      elt.find("i").on("click", removeExtracted.bind(null, extracted.iban, "iban", i));
      i++;
      $('#ibanTable > tbody:last-child').append(elt);
    });
    $('#metaHeader').show();
    $('#iban-div').show();
  }
}
