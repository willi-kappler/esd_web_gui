console.log("grain_outline.js");
var images;
var canvases;
var num_of_images;
var bw_threshold;

window.addEventListener("load", function(){
    console.log("loaded");

    images = document.getElementsByName("grain_image");
    canvases = document.getElementsByName("grain_canvas");

    if (images && canvases) {
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

function blur_image1(pixel_data) {
  /*
  filter_image(pixel_data, [
    0, 0, 0, 5, 0, 0, 0,
    0, 5, 18, 32, 18, 5, 0,
    0, 18, 64, 100, 64, 18, 0,
    5, 32, 100, 100, 100, 32, 5,
    0, 18, 64, 100, 64, 18, 0,
    0, 5, 18, 32, 18, 5, 0,
    0, 0, 0, 5, 0, 0, 0
  ]);
*/

  filter_image(pixel_data, [
    0.0, 0.0, 0.0, 0.0046816479400749065, 0.0, 0.0, 0.0,
    0.0, 0.0046816479400749065, 0.016853932584269662, 0.0299625468164794, 0.016853932584269662, 0.0046816479400749065, 0.0,
    0.0, 0.016853932584269662, 0.0599250936329588, 0.09363295880149813, 0.0599250936329588, 0.016853932584269662, 0.0,
    0.0046816479400749065, 0.0299625468164794, 0.09363295880149813, 0.09363295880149813, 0.09363295880149813, 0.0299625468164794, 0.0046816479400749065,
    0.0, 0.016853932584269662, 0.0599250936329588, 0.09363295880149813, 0.0599250936329588, 0.016853932584269662, 0.0,
    0.0, 0.0046816479400749065, 0.016853932584269662, 0.0299625468164794, 0.016853932584269662, 0.0046816479400749065, 0.0,
    0.0, 0.0, 0.0, 0.0046816479400749065, 0.0, 0.0, 0.0
  ]);

}

function laplace_image(pixel_data) {
  filter_image(pixel_data, [
    -1, -1, -1,
    -1, 8, -1,
    -1, -1, -1
  ]);
}

function redraw_image(image_index) {
  if (images && canvases) {
    if (image_index >= 0 && image_index < num_of_images) {
      var context = canvases[image_index].getContext("2d");
      if (context) {
        context.drawImage(images[image_index], 0, 0);
        var pixel_data = context.getImageData(0, 0, images[image_index].width, images[image_index].height);

        blur_image1(pixel_data);
        bw_image(pixel_data, image_index);
        laplace_image(pixel_data);

        context.putImageData(pixel_data, 0, 0);
      }
    }
  }
}

function filter_image(pixel_data, weights) {
  var side = Math.round(Math.sqrt(weights.length));
  var halfSide = Math.floor(side/2);
  var src = pixel_data.data;
  var sw = pixel_data.width;
  var sh = pixel_data.height;
  // Pad output by the convolution matrix
  var w = sw;
  var h = sh;
  // Temporary canvas for output data:
  var tmpCanvas = document.createElement("canvas");
  var tmpCtx = tmpCanvas.getContext("2d");
  var output = tmpCtx.createImageData(w, h);
  var dst = output.data;

  // Go through the destination image pixels
  for (var y = 0; y < h; y++) {
    for (var x = 0; x < w; x++) {
      var sy = y;
      var sx = x;
      var dstOff = (y * w + x) * 4;
      // Calculate the weighed sum of the source image pixels that
      // Fall under the convolution matrix
      var r = 0, g = 0, b = 0;
      for (var cy = 0; cy < side; cy++) {
        for (var cx = 0; cx < side; cx++) {
          var scy = sy + cy - halfSide;
          var scx = sx + cx - halfSide;
          if (scy >= 0 && scy < sh && scx >= 0 && scx < sw) {
            var srcOff = (scy * sw + scx) * 4;
            var wt = weights[cy * side + cx];
            r += src[srcOff] * wt;
            g += src[srcOff + 1] * wt;
            b += src[srcOff + 2] * wt;
          }
        }
      }
      dst[dstOff] = r;
      dst[dstOff + 1] = g;
      dst[dstOff + 2] = b;
      dst[dstOff + 3] = 255;
    }
  }

  for (var i = 0; i < pixel_data.data.length; i++) {
    pixel_data.data[i] = dst[i];
    // pixel_data.data[i] = 128;
  }
}

function bw_image(pixel_data, image_index) {
  var orig_value;

  for (var i = 0; i < pixel_data.data.length; i += 4) {
    orig_value = (pixel_data.data[i] + pixel_data.data[i + 1] + pixel_data.data[i + 2]) / (255.0 * 3.0);

    if (orig_value < bw_threshold[image_index]) {
        pixel_data.data[i] = 0;
        pixel_data.data[i + 1] = 0;
        pixel_data.data[i + 2] = 0;
        pixel_data.data[i + 3] = 255;
    } else {
      pixel_data.data[i] = 255;
      pixel_data.data[i + 1] = 255;
      pixel_data.data[i + 2] = 255;
      pixel_data.data[i + 3] = 0;
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
