console.log("grain_outline.js");
var images;
var canvases;
var num_of_images;
var bw_threshold;

window.addEventListener("load", function(){
    console.log("loaded");

    images = document.getElementsByName("grain_image");
    canvases = document.getElementsByName("grain_canvas");

    if (images) {
      num_of_images = images.length;
      console.log("number of images: " + num_of_images);

      if (!bw_threshold) {
        bw_threshold = [];
        for (var i = 0; i < num_of_images; i++) {
          bw_threshold.push(0.5);
        }
      }

      for (var i = 0; i < num_of_images; i++) {
        canvases[i].width = images[i].width;
        canvases[i].height = images[i].height;

        redraw_image(i);
      }
      console.log("image processing finished");
    }
});

function redraw_image(image_index) {
  if (images) {
    if (image_index >= 0 && image_index < num_of_images) {
      var context = canvases[image_index].getContext('2d');
      context.drawImage(images[image_index], 0, 0);
      var pixel_data = context.getImageData(0, 0, images[image_index].width, images[image_index].height);

      bw_image(pixel_data, image_index);

      context.putImageData(pixel_data, 0, 0);
    }
  }
}

function blur_image(pixel_data, width, height) {
  var pixel_value;

  for (y = 1; y < height - 1; y++) {
    for (x = 1; x < width - 1; x++) {
      pixel_value = pixel_data.data[((y - 1) * (width * 4)) + ((x - 1) * 4)];
      pixel_value += pixel_data.data[((y - 1) * (width * 4)) + (x * 4)];
      pixel_value += pixel_data.data[((y - 1) * (width * 4)) + ((x + 1) * 4)];

      pixel_value += pixel_data.data[(y * (width * 4)) + ((x - 1) * 4)];
      pixel_value += pixel_data.data[(y * (width * 4)) + (x * 4)];
      pixel_value += pixel_data.data[(y * (width * 4)) + ((x + 1) * 4)];

      pixel_value = pixel_data.data[((y + 1) * (width * 4)) + ((x - 1) * 4)];
      pixel_value += pixel_data.data[((y + 1) * (width * 4)) + (x * 4)];
      pixel_value += pixel_data.data[((y + 1) * (width * 4)) + ((x + 1) * 4)];

      // TODO: store pixel value to destination array
    }
  }
}

function bw_image(pixel_data, image_index) {
  var orig_value;

  for (var j = 0; j < pixel_data.data.length; j += 4) {
    orig_value = (pixel_data.data[j] + pixel_data.data[j + 1] + pixel_data.data[j + 2]) / (255.0 * 3.0);

    if (orig_value < bw_threshold[image_index]) {
        pixel_data.data[j] = 0;
        pixel_data.data[j + 1] = 0;
        pixel_data.data[j + 2] = 0;
    } else {
      pixel_data.data[j] = 255;
      pixel_data.data[j + 1] = 255;
      pixel_data.data[j + 2] = 255;
      pixel_data.data[j + 3] = 0;
    }
  }
}

function inc_bw_threshold(image_index) {
  if (images) {
    if (image_index >= 0 && image_index < num_of_images) {
      bw_threshold[image_index] += 0.05;
      if (bw_threshold[image_index] > 1.0) {
        bw_threshold[image_index] = 1.0
      }

      redraw_image(image_index);
    }
  }
}

function dec_bw_threshold(image_index) {
  if (images) {
    if (image_index >= 0 && image_index < num_of_images) {
      bw_threshold[image_index] -= 0.05;
      if (bw_threshold[image_index] < 0.0) {
        bw_threshold[image_index] = 0.0
      }

      redraw_image(image_index);
    }
  }
}
