#!/usr/bin/env python3
"""Real offscreen CEF regression, using a caller-provided pinned runtime."""
import argparse
import json
import os
from pathlib import Path
import socket
import struct
import subprocess
import tempfile
import time


def exact(stream, size):
    result = bytearray()
    while len(result) < size:
        part = stream.recv(size - len(result))
        assert part, 'Web host disconnected'
        result.extend(part)
    return result


def run(host, runtime):
    parent, child = socket.socketpair()
    parent.settimeout(12)
    env = dict(os.environ, LD_LIBRARY_PATH=str(runtime), VARPAPER_CEF_RUNTIME=str(runtime))
    process = subprocess.Popen([str(host), f'--ipc-fd={child.fileno()}'], env=env, pass_fds=[child.fileno()], start_new_session=True)
    child.close()
    request = 0
    def call(op, ident=0, **values):
        nonlocal request
        request += 1
        command = dict(op=op, id=ident, generation=ident, request=request, **values)
        data = json.dumps(command).encode()
        parent.sendall(struct.pack('<I', len(data)) + data)
        size = struct.unpack('<I', exact(parent, 4))[0]
        assert 0 < size <= 65536
        response = json.loads(exact(parent, size))
        assert response['request'] == request and response['id'] == ident
        assert response['generation'] == ident
        pixels = exact(parent, response['bytes'])
        return response, pixels
    def ok(op, ident=0, **values):
        response, pixels = call(op, ident, **values)
        assert response['ok'], response
        return response, pixels
    def frame(ident, width=32, height=24, sequence=0):
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline:
            response, pixels = ok('frame', ident, width=width, height=height, sequence=sequence)
            if pixels:
                assert len(pixels) == width*height*4
                return response, pixels
            time.sleep(.02)
        raise AssertionError('No Web first frame')
    try:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root/'style.css').write_text('html,body{margin:0;background:rgb(20,40,80)}canvas{display:block}')
            (root/'animate.js').write_text("const c=document.querySelector('canvas'),x=c.getContext('2d');c.width=innerWidth;c.height=innerHeight;let n=0;setInterval(()=>{n++;x.fillStyle='rgb('+((n%2)?200:100)+',60,30)';x.fillRect(0,0,c.width,c.height)},100);")
            (root/'index page.html').write_text('<link rel="stylesheet" href="style.css"><canvas></canvas><script src="animate.js"></script>')
            (root/'dot.svg').write_text('<svg xmlns="http://www.w3.org/2000/svg" width="1" height="1"><rect width="1" height="1" fill="rgb(200,160,10)"/></svg>')
            (root/'blue.html').write_text('<body style="margin:0;background:rgb(10,20,200)"><img src="dot.svg" style="position:absolute;left:1px;top:1px">')
            (root/'worker.js').write_text('try{new WebSocket("wss://example.com/blocked")}catch(e){}')
            (root/'worker.html').write_text('<script>new Worker("worker.js");</script>')
            (root/'socket.html').write_text('<script>try{new WebSocket("wss://example.com/blocked")}catch(e){}</script>')
            (root/'rtc.html').write_text('<script>try{new RTCPeerConnection({iceServers:[{urls:"stun:example.com:3478"}]})}catch(e){}</script>')
            (root/'dialog.html').write_text('<script>alert("Wallpaper must stay windowless");</script>')
            (root/'asset-link.js').symlink_to('/etc/passwd')
            (root/'symlink.html').write_text('<script>fetch("asset-link.js").catch(()=>{});</script>')
            (root/'download.html').write_text('<script>const a=document.createElement("a");a.href="blue.html";a.download="wallpaper.html";a.click();</script>')
            (root/'protocol.html').write_text('<script>location.href="varpaper-test://launch";</script>')
            (root/'remote.html').write_text('<script>fetch("https://example.com/blocked").catch(()=>{});</script>')
            (root/'webgl.html').write_text('<body style="margin:0"><canvas></canvas><script>const c=document.querySelector("canvas");c.width=innerWidth;c.height=innerHeight;const g=c.getContext("webgl");g.clearColor(0.1,0.8,0.2,1);g.clear(g.COLOR_BUFFER_BIT);</script>')
            (root/'escape.html').write_text('<script>fetch("file:///etc/passwd").catch(()=>{});</script>')
            ok('create', 1, root=str(root), entry='index page.html', width=32, height=24)
            first, pixels = frame(1)
            assert pixels[3] == 255 and pixels[1] == 60, pixels[:4]
            next_frame, changed = frame(1, sequence=first['sequence'])
            deadline=time.monotonic()+2
            while changed == pixels and time.monotonic()<deadline:
                next_frame,changed=frame(1,sequence=next_frame['sequence'])
            assert next_frame['sequence'] > first['sequence']
            assert changed != pixels, 'Canvas did not animate'
            (root/'BLUE.HTML').write_text((root/'blue.html').read_text())
            ok('create', 2, root=str(root), entry='BLUE.HTML', width=32, height=24)
            _, blue = frame(2)
            assert blue[:4] == bytes([200,20,10,255]), blue[:4]
            offset=(32+1)*4
            assert blue[offset:offset+4] == bytes([10,160,200,255]), blue[offset:offset+4]
            resized, _ = frame(1, width=40, height=30, sequence=next_frame['sequence'])
            ok('pause', 1)
            paused, _ = ok('frame', 1, width=40, height=30, sequence=resized['sequence'])
            time.sleep(.25)
            stopped, still = ok('frame', 1, width=40, height=30, sequence=paused['sequence'])
            assert stopped['sequence'] == paused['sequence'] and not still
            ok('resume', 1)
            frame(1, width=40, height=30, sequence=paused['sequence'])
            ok('close', 1)
            frame(2)
            for ident, name in [(3,'remote.html'), (4,'escape.html'), (8,'download.html'), (9,'protocol.html'), (10,'symlink.html'), (12,'socket.html'), (13,'rtc.html'), (14,'worker.html'), (15,'dialog.html')]:
                ok('create',ident,root=str(root),entry=name,width=32,height=24)
                deadline = time.monotonic()+10
                while time.monotonic()<deadline:
                    response, _ = call('frame',ident,width=32,height=24,sequence=0)
                    if not response['ok']:
                        assert any(word in response['error'].lower() for word in ['blocked','escapes']), response
                        break
                    time.sleep(.02)
                else:
                    raise AssertionError(f'Forbidden resource request was not diagnosed for {ident}: {name}')
                ok('close',ident)
            response,_=call('create',5,root=str(root),entry='../outside',width=32,height=24)
            assert not response['ok']
            response,_=call('create',11,root=str(root),entry='blue.html',width=100000,height=100000)
            assert not response['ok'] and 'budget' in response['error']
            ok('create',7,root=str(root),entry='webgl.html',width=32,height=24)
            _, green=frame(7)
            assert abs(green[1]-204) <= 2 and abs(green[0]-51) <= 2, green[:4]
            ok('close',7)
            ok('close',2)
            ok('shutdown')
        process.wait(timeout=10)
        assert process.returncode == 0, process.returncode
        # The CEF process group must not retain renderer/GPU/utility children.
        try:
            os.killpg(process.pid,0)
        except ProcessLookupError:
            pass
        else:
            raise AssertionError('CEF subprocess group survived shutdown')
        print('Web host: local resources, Canvas/WebGL, independent browsers, resize, pause/resume, isolation and cleanup passed')
    finally:
        parent.close()
        if process.poll() is None:
            os.killpg(process.pid,9)
            process.wait()


if __name__ == '__main__':
    parser=argparse.ArgumentParser()
    parser.add_argument('--host',type=Path,required=True)
    parser.add_argument('--runtime',type=Path,required=True)
    options=parser.parse_args()
    run(options.host.resolve(),options.runtime.resolve())
