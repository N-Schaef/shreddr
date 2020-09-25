
$('#matcherType').on('change', function() {
  if(this.value == "1"){
    $( "#fullMatcher" ).show();
    $( "#anyMatcher" ).hide();
    $( "#Regex" ).hide();
  }else if(this. value == "2"){
    $( "#fullMatcher" ).hide();
    $( "#anyMatcher" ).hide();
    $( "#Regex" ).show();
  }else if(this. value == "3"){
    $( "#fullMatcher" ).hide();
    $( "#anyMatcher" ).show();
    $( "#Regex" ).hide();
  }else if(this. value == "4"){
    $( "#fullMatcher" ).hide();
    $( "#anyMatcher" ).hide();
    $( "#Regex" ).hide();
  }
});

$('#matcherType').trigger("change");