
$('#matcherType').on('change', function() {
  if(this.value == "1"){
    $( "#fullMatcher" ).show();
    $( "#Regex" ).hide();
  }else if(this. value == "2"){
    $( "#fullMatcher" ).hide();
    $( "#Regex" ).show();
  }
});
