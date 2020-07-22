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

function createTagButton(docId,tagMap, tagId, clickFunc) {
  var tag = tagMap.get(tagId);
  if (!tag) {
    return "";
  }
  var btn = $("<a href=\"/tags/"+tagId+"/edit\" type=\"button\" class=\"btn  btn-sm tag-btn mb-2\"></a>");
  btn.text(tag.name);
  btn.css("background-color", tag.color);
  btn.addClass("tag-" + tag.id);
  btn.on("click", clickFunc);
  var removebtn = $("<button type=\"button\" class=\"btn  btn-sm tag-btn mb-2\"><span data-feather=\"x\"></span></button>");
  removebtn.css("background-color", tag.color);
  removebtn.on("click", function(){
    $.ajax({
      url: '/api/documents/'+docId+'/tags/'+tag.id,
      type: 'DELETE',
      success: function(result) {
        location.reload();
      }
  });
  }.bind(tag));
  var btnGroup = $("<div class=\"btn-group mr-2\"></div>")
  btnGroup.append(btn);
  btnGroup.append(removebtn);
  return btnGroup;
}

function createAddButton(docId, tagMap){
  var div = $("<div></div>");
  var dropdownButton = $("<button class=\"btn btn-sm btn-success dropdown-toggle\" type=\"button\" id=\"addTagButton\" data-toggle=\"dropdown\"><span data-feather=\"plus\"></span></button>");
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
      $.ajax({
        url: '/api/documents/'+docId+'/tags/'+value.id,
        type: 'PUT',
        success: function(result) {
          location.reload();
        }
    });
    }.bind(docId,value));
    items.append(item);
  });
  div.append(dropdownButton);
  div.append(items);
  return div;
}

function createTagButtons(docId,tags){
  $.get("/api/tags")
  .done(function (data) {
    let tagMap = new Map()
    data.forEach(function (tag) {
      tagMap.set(tag.id, tag);
    });
    tags.forEach(function(tag){
      $("#tags").append(createTagButton(docId,tagMap,tag,function(){}));
      tagMap.delete(tag);
    });
    $("#tags").append(createAddButton(docId,tagMap));

    feather.replace()
  });
}

$("#edit-title-btn").on("click", function(){
  if($("#title").is(":visible")){
    $("#title").hide();
    $("#titleForm").show();
  }else{
    $("#title").show();
    $("#titleForm").hide();
  }

});


$("#edit-lang-btn").on("click", function(){
  if($("#lang").is(":visible")){
  $("#lang").hide();
  $("#langForm").show();
  }else{
    $("#lang").show();
    $("#langForm").hide();
  }
});

$('#datepicker').datepicker();
$('#datepicker').on('changeDate', function() {
  $('#docDateInput').val(
      $('#datepicker').datepicker('getUTCDate').getTime()/1000
  );
});