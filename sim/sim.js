var scene, camera, renderer, clock, controls, composer, listener;

var spines;

// Camera presets:
const camera1_pos = [0.8, -1.5, 1.1];
const camera1_target = {x: 0, y: 0, z: 1.1};
const camera2_pos = [0, 3, 1.1];
const camera2_target = {x: 0, y: 0, z: 1.1};
const camera3_pos = [0, 0, 5];
const camera3_target = {x: 0, y: 0, z: 1.1};

function init_world() {
    scene = new THREE.Scene();

    renderer = new THREE.WebGLRenderer();
    renderer.setSize(window.innerWidth, window.innerHeight);
    renderer.autoClear = false;

    camera = new THREE.PerspectiveCamera(
        75, window.innerWidth/window.innerHeight, 0.1, 1000);
    camera.position.set(camera1_pos[0], camera1_pos[1], camera1_pos[2]);
    camera.lookAt(camera1_target);

    listener = new THREE.AudioListener();
    camera.add(listener);

    clock = new THREE.Clock();

    controls = new THREE.FlyControls(camera);
    controls.domElement = renderer.domElement;
    controls.rollSpeed = 1;
    controls.movementSpeed = 3;
    controls.dragToLook = true;

    composer = new THREE.EffectComposer(renderer);
    var renderPass = new THREE.RenderPass(scene, camera);
    composer.addPass(renderPass);
    var bloomPass = new THREE.BloomPass(1.5);
    var copyPass = new THREE.ShaderPass(THREE.CopyShader);
    copyPass.renderToScreen = true;
    //composer.addPass(bloomPass);
    composer.addPass(copyPass);

    document.body.appendChild(renderer.domElement);
}

function init_scene() {
    var alight = new THREE.AmbientLight(0x333333);
    scene.add(alight);

    var sky_geo = new THREE.BoxGeometry(100, 100, 100);
    var sky_mat = new THREE.MeshLambertMaterial(
        { color: 0x000020, side: THREE.BackSide });
    var sky_msh = new THREE.Mesh(sky_geo, sky_mat);
    scene.add(sky_msh);

    var gnd_geo = new THREE.PlaneGeometry(16, 16, 128, 128);
    var gnd_mat = new THREE.MeshLambertMaterial({color: 0x40ff40});
    var gnd_msh = new THREE.Mesh(gnd_geo, gnd_mat);
    scene.add(gnd_msh);

    var pole_geo = new THREE.CylinderGeometry(.005, .005, 1.1);
    var pole_mat = new THREE.MeshLambertMaterial({
        color: 0x666666,
        emissive: 0x333333,
        emissiveIntensity: 1,
    });

    var centre_geo = new THREE.SphereGeometry(0.1, 32, 16);
    var centre_mat = new THREE.MeshLambertMaterial({
        color: 0x666666,
        emissive: 0x666666,
        emissiveIntensity: 1,
    });
    var centre = new THREE.Mesh(centre_geo, centre_mat);
    centre.position.set(0, 0, 1.1);
    scene.add(centre);

    var led_geo = new THREE.SphereGeometry(0.007, 8, 8);
    var led_mat = new THREE.MeshLambertMaterial({
        color: 0x666666,
        emissive: 0xFFFFFF,
        emissiveIntensity: 1,
    });

    // X is distance right of the starting point
    // Y is distance away from the camera starting position
    // Z is height above the floor

    // Each of the following vertices is the 3d coordinates of a 5-hub on the
    // outer mesh, and the 3d rotations required to make the spine the correct
    // angle.
    const phi = 1.618;
    const vertex_locations = [
        [[0, 1, phi],   [Math.PI / 2 - Math.atan(1.0 / phi), 0, 0]],
        [[0, 1, -phi],  [Math.PI / 2 + Math.atan(1.0 / phi), 0, 0]],
        [[0, -1, phi],  [Math.PI / 2 + Math.atan(1.0 / phi), 0, 0]],
        [[0, -1, -phi], [Math.PI / 2 - Math.atan(1.0 / phi), 0, 0]],
        [[1, phi, 0],   [0, 0, -Math.atan(1.0 / phi)]],
        [[1, -phi, 0],  [0, 0, Math.atan(1.0 / phi)]],
        [[-1, phi, 0],  [0, 0, Math.atan(1.0 / phi)]],
        [[-1, -phi, 0], [0, 0, -Math.atan(1.0 / phi)]],
        [[phi, 0, 1],   [0, -Math.atan(1.0 / phi), Math.PI / 2]],
        [[phi, 0, -1],  [0, Math.atan(1.0 / phi), Math.PI / 2]],
        [[-phi, 0, 1],  [0, Math.atan(1.0 / phi), Math.PI / 2]],
        [[-phi, 0, -1], [0, -Math.atan(1.0 / phi), Math.PI / 2]],
    ];

    // Axis markers - useful for figuring out where the world axes line up relative to the LEDs.
    // for(var i = 0; i < 100; i++) {
    //     scale = 30;
    //     var marker_mesh = new THREE.SphereGeometry(0.02, 8, 8);

    //     var marker_mat_x = new THREE.MeshLambertMaterial({ emissive: 0xff0000, emissiveIntensity: 1 });
    //     var mesh_x = new THREE.Mesh(marker_mesh, marker_mat_x);
    //     mesh_x.position.set(i / scale, 0, 1.1);
    //     scene.add(mesh_x);

    //     var marker_mat_y = new THREE.MeshLambertMaterial({ emissive: 0x00ff00, emissiveIntensity: 1 });
    //     var mesh_y = new THREE.Mesh(marker_mesh, marker_mat_y);
    //     mesh_y.position.set(0, i / scale, 1.1);
    //     scene.add(mesh_y);

    //     var marker_mat_z = new THREE.MeshLambertMaterial({ emissive: 0x0000ff, emissiveIntensity: 1 });
    //     var mesh_z = new THREE.Mesh(marker_mesh, marker_mat_z);
    //     mesh_z.position.set(0, 0, 1.1 + i / scale);
    //     scene.add(mesh_z);
    // }

    spines = [];
    for(var i = 0; i < vertex_locations.length; i++) {
        var spine = new THREE.Mesh(pole_geo.clone(), pole_mat.clone());
        var light = new THREE.PointLight(0x000000, 1, 1);

        const scaling = 1.1 / 1.9 / 2;
        spine.rotation.set(vertex_locations[i][1][0],
                           vertex_locations[i][1][1],
                           vertex_locations[i][1][2]);
        // This position is the centre of the spine cylinder, so it wants to be half the vertex
        spine.position.set(vertex_locations[i][0][0] * scaling,
                           vertex_locations[i][0][1] * scaling,
                           vertex_locations[i][0][2] * scaling + 1.1);

        light.position.set(0, -1.2, 0);
        spine.add(light);
        scene.add(spine);
        spines[i] = spine;

        // Place LEDs along the spine
        spines[i].leds = []
        for (var j = 0; j < 59; j++) {
            var led = new THREE.Mesh(led_geo.clone(), led_mat.clone());
            const led_scaling = 1.1 / 1.9 * j / 60.0;
            led.position.set(vertex_locations[i][0][0] * led_scaling,
                             vertex_locations[i][0][1] * led_scaling,
                             vertex_locations[i][0][2] * led_scaling + 1.1)
            scene.add(led);
            spines[i].leds[j] = led;
        }
    }
}

function set_led(spine_num, led_num, data) {
    var spine = spines[spine_num];
    var led = spine.leds[led_num];
    var light = spine.children[0];
    var color = new THREE.Color(data[0]/255, data[1]/255, data[2]/255);
    led.material.emissive.setHex(color.getHex());
    //light.color.setHex(color.getHex());
}

function on_window_resize(event) {
    renderer.setSize(window.innerWidth, window.innerHeight);
    composer.setSize(window.innerWidth, window.innerHeight);
    camera.aspect = window.innerWidth / window.innerHeight;
    camera.updateProjectionMatrix();
}

function on_keypress(event) {
    var code = event.code;
    if(code == "Digit1") {
        camera_pos = camera1_pos;
        camera_target = camera1_target;
        camera.position.set(camera_pos[0], camera_pos[1], camera_pos[2]);
        camera.lookAt(camera_target);
    } else if(code == "Digit2") {
        camera_pos = camera2_pos;
        camera_target = camera2_target;
        camera.position.set(camera_pos[0], camera_pos[1], camera_pos[2]);
        camera.lookAt(camera_target);
    } else if(code == "Digit3") {
        camera_pos = camera3_pos;
        camera_target = camera3_target;
        camera.position.set(camera_pos[0], camera_pos[1], camera_pos[2]);
        camera.lookAt(camera_target);
    }
}

function render() {
    var delta = clock.getDelta();
    controls.update(delta);
    composer.render();
}

function update() {
}

var ws;
var ws_path = "ws://localhost:3030/ws";
function init_ws() {
    ws = new WebSocket(ws_path);
    ws.onclose = retry_ws;
    ws.onerror = retry_ws;
    ws.onmessage = handle_ws;
}

function changeHost(new_host) {
    ws_path = "ws://" + new_host + ":3030/ws";
    ws.close();
    retry_ws();
}

function handle_ws(event) {
    var status = document.getElementById('status');
    status.style.color = 'green';
    status.innerHTML = 'Connected';
    var spineData = JSON.parse(event.data).spines;
    for(var spine = 0; spine < 12; spine++) { // spine
        for(var led = 0; led < 59; led++) { // led
            set_led(spine, led, spineData[spine][led]);
        }
    }
}

function retry_ws() {
    ws.close();
    console.log("Websocket closed/error, retrying in 1s");
    var status = document.getElementById('status');
    status.style.color = 'red';
    status.innerHTML = 'Disconnected';
    window.setTimeout(function() {
        // Clean up the old websocket before closing down making a new one
        ws.onclose = null;
        ws.onmessage = null;
        ws.close();

        ws = new WebSocket(ws_path);
        ws.onclose = retry_ws;
        ws.onmessage = handle_ws;
    }, 1000);
}

var stats;
function init_stats() {
    stats = new Stats();
    document.body.appendChild(stats.dom);
}

function init() {
    window.addEventListener('resize', on_window_resize, false);
    window.addEventListener('keypress', on_keypress, false);
    init_world();
    init_scene();
    init_ws();
    init_stats();
}

function animate() {
    requestAnimationFrame(animate);
    stats.begin();
    update();
    render();
    stats.end();
}

init();
animate();
