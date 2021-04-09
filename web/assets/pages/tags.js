(function () {
  'use strict'

  $.get("/tags/json")
  .done(function( data ) {
    $.each(data, function () {
      if (this.deactivated) {
        return;
      }

      var newTag = $("template.tag").contents().clone();
      newTag.find(".card-header").append(this.name);
      newTag.find(".edit-tag").attr("href",`/tags/${this.id}`);
      newTag.find(".remove-tag").on('click', function(){
          $.ajax({
            url: '/tags/'+this.id,
            type: 'DELETE',
            success: function(result) {
              location.reload();
            }
          });
        }.bind(this)
      );

      if (this.color != null) {
        var header = newTag.find(".card-header");
        header.css('background-color', this.color);
        header.css('color', isDark(this.color) ? "var(--light)" : "var(--dark");
      }

      newTag.show();
      $(newTag).appendTo("#tags");
      feather.replace(); // Fill icon placeholders
    });
  });
})()
