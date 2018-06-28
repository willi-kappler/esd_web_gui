console.log("grain_outline.js");

window.addEventListener("load", function(){
    console.log("loaded");

    var images = document.getElementsByName("grain_images");
    if (images) {
      console.log("number of images: " + images.length);
      for (var image of images) {
          console.log("width: " + image.width + ", height: " + image.height);
      }
    }

});
