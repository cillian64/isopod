// Render the base map.  Call the provided callback after the base map has loaded and drawn
function renderBaseMap(callback) {
    // First, load the base map image
    const image = document.createElement("img");
    image.src = "emf_base_map.png";
    // After the base map image loads...
    image.addEventListener("load", () => {
        const imageWidth = image.width;
        const imageHeight = image.height;

        // Resize the canvas to match the base map image
        const canvasElement = document.getElementById("canvas");
        canvasElement.width = imageWidth;
        canvasElement.height = imageHeight;

        // Draw the image on the canvas
        const context = canvasElement.getContext("2d");
        context.drawImage(image, 0, 0, imageWidth, imageHeight);

        callback();
    });
}

// Convert a decimal latitude and longitude to map pixel coordinates
function latLongToPix(lat, long) {
    // Calibration values worked out using google maps:
    // Bottom left of map is:
    const bottomLeft = [52.038889, -2.380556]; // Decimal degrees
    const topRight = [52.044046, -2.374030];   // Decimal degrees
    const mapSize = [994, 1275];               // Pixels

    const xPixPerDegree = mapSize[0] / (topRight[1] - bottomLeft[1])
    const yPixPerDegree = mapSize[1] / (bottomLeft[0] - topRight[0])
    const xDegreeOffset = bottomLeft[1]
    const yDegreeOffset = topRight[0]

    const xPix = (long - xDegreeOffset) * xPixPerDegree
    const yPix = (lat - yDegreeOffset) * yPixPerDegree

    return [xPix, yPix]
}

// A list of lat-long pairs, each indicating a point on the path, the last
// indicating the most-recent-known location.
const isopodPath = [
    [52.041533, -2.378381],
    [52.043078, -2.376879],
    [52.043071, -2.375012],
    [52.041368, -2.375227],
    [52.039771, -2.375967],
];

function renderPath(isopodPath) {
    const canvasElement = document.getElementById("canvas");
    const context = canvasElement.getContext("2d");

    if (isopodPath.length < 2) {
        return;
    }

    context.beginPath();
    const locationPix = latLongToPix(isopodPath[0][0], isopodPath[0][1]);
    context.moveTo(locationPix[0], locationPix[1]);
    context.lineWidth = 5;
    context.strokeStyle = "black";
    context.lineJoin = "round";
    context.lineCap = "round";

    for (let i = 1; i < isopodPath.length; i++) {
        const locationPix = latLongToPix(isopodPath[i][0], isopodPath[i][1]);
        context.lineTo(locationPix[0], locationPix[1]);
    }
    context.stroke();
}

function renderCross(context, x, y) {
    context.beginPath();
    context.lineWidth = 10;
    context.strokeStyle = "black";
    context.lineJoin = "round";
    context.lineCap = "round";
    context.moveTo(x - 10, y - 10);
    context.lineTo(x + 10, y + 10);
    context.stroke();
    context.beginPath();
    context.moveTo(x + 10, y - 10);
    context.lineTo(x - 10, y + 10);
    context.stroke();
}

function renderCurrentLocation(isopodLocation) {
    const canvasElement = document.getElementById("canvas");
    const context = canvasElement.getContext("2d");
    const locationPix = latLongToPix(isopodLocation[0], isopodLocation[1]);
    renderCross(context, locationPix[0], locationPix[1]);
}


renderBaseMap(() => {
    renderPath(isopodPath);
    renderCurrentLocation(isopodPath[isopodPath.length - 1]);
});
