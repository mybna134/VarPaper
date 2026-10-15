#!/usr/bin/env python3
"""Real offscreen CEF regression, using a caller-provided pinned runtime."""
import argparse
import json
import math
import os
from pathlib import Path
import socket
import struct
import subprocess
import tempfile
import time
import wave


def exact(stream, size):
    result = bytearray()
    while len(result) < size:
        part = stream.recv(size - len(result))
        assert part, 'Web host disconnected'
        result.extend(part)
    return result

def descendants(root, host):
    parents={}
    for proc in Path('/proc').iterdir():
        if not proc.name.isdecimal(): continue
        try:
            parents[int(proc.name)]=int((proc/'stat').read_text().rpartition(')')[2].split()[1])
        except (PermissionError,FileNotFoundError,ProcessLookupError): pass
    owned={root}
    if Path('/.flatpak-info').is_file():
        # Portal-spawned Chromium children are not OS descendants of the host.
        # The private Flatpak PID namespace contains this fixture only.
        for pid in parents:
            try:
                args = (Path('/proc') / str(pid) / 'cmdline').read_bytes().split(b'\0')
                if args[0] == os.fsencode(host):
                    owned.add(pid)
            except (PermissionError, FileNotFoundError, ProcessLookupError):
                pass
    while True:
        children={pid for pid,parent in parents.items() if parent in owned}
        if children.issubset(owned): return owned-{root}
        owned.update(children)


def run(host, runtime):
    parent, child = socket.socketpair()
    parent.settimeout(12)
    env = dict(os.environ, LD_LIBRARY_PATH=str(runtime), VARPAPER_CEF_RUNTIME=str(runtime), SDL_AUDIODRIVER='dummy')
    command = [str(host), f'--ipc-fd={child.fileno()}']
    if Path('/.flatpak-info').is_file():
        env['ZYPAK_CEF_LIBRARY_PATH'] = str(runtime / 'libcef.so')
        command.insert(0, '/app/bin/zypak-wrapper')
    process = subprocess.Popen(command, env=env, pass_fds=[child.fileno()], start_new_session=True)
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
        assert response['ipc_version'] == 2
        pixels = exact(parent, response['bytes'])
        return response, pixels
    def ok(op, ident=0, **values):
        response, pixels = call(op, ident, **values)
        assert response['ok'], response
        return response, pixels
    def frame(ident, width=32, height=24, sequence=0, ready=None, **values):
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline:
            response, pixels = ok('frame', ident, width=width, height=height, sequence=sequence, **values)
            if pixels:
                assert len(pixels) == width*height*4
                if ready is None or ready(pixels):
                    return response, pixels
                sequence = response['sequence']
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
            (root/'callbacks.html').write_text('''<body style="margin:0"><script>
            let count=0;
            window.wallpaperPropertyListener={applyUserProperties(p){
              if(typeof p.enabled.value!=='boolean'||typeof p.speed.value!=='number'||typeof p.caption.value!=='string')throw Error('Incorrect property types');
              count++;
            }};
            wallpaperRegisterAudioListener(a=>{
              if(!Array.isArray(a)||a.length!==128||!a.every(Number.isFinite))throw Error('Incorrect audio callback');
              document.body.style.background='rgb('+(40*count)+','+Math.round(a[0]*200)+','+Math.round(a[64]*200)+')';
            });</script>''')
            (root/'sound.html').write_text('''<script>
            const ctx=new AudioContext(),osc=ctx.createOscillator(),gain=ctx.createGain();
            gain.gain.value=.5;osc.frequency.value=375;
            osc.connect(gain);gain.connect(ctx.destination);osc.start();ctx.resume();
            </script>''')
            with wave.open(str(root/'tone.wav'),'wb') as sound:
                sound.setnchannels(1);sound.setsampwidth(2);sound.setframerate(48000)
                sound.writeframes(b''.join(struct.pack('<h',round(16383*math.sin(2*math.pi*375*n/48000))) for n in range(48000)))
            (root/'media-sound.html').write_text('<audio src="tone.wav" autoplay loop></audio>')
            (root/'asset-link.js').symlink_to('/etc/passwd')
            (root/'symlink.html').write_text('<script>fetch("asset-link.js").catch(()=>{});</script>')
            (root/'download.html').write_text('<script>const a=document.createElement("a");a.href="blue.html";a.download="wallpaper.html";a.click();</script>')
            (root/'protocol.html').write_text('<script>location.href="varpaper-test://launch";</script>')
            (root/'remote.html').write_text('<script>fetch("https://example.com/blocked").catch(()=>{});</script>')
            (root/'webgl.html').write_text('<body style="margin:0"><canvas></canvas><script>const c=document.querySelector("canvas");c.width=innerWidth;c.height=innerHeight;const g=c.getContext("webgl");g.clearColor(0.1,0.8,0.2,1);g.clear(g.COLOR_BUFFER_BIT);</script>')
            (root/'escape.html').write_text('<script>fetch("file:///etc/passwd").catch(()=>{});</script>')
            ok('create', 1, root=str(root), entry='index page.html', width=32, height=24)
            first, pixels = frame(1, ready=lambda pixels: pixels[3] == 255 and pixels[1] == 60)
            assert pixels[3] == 255 and pixels[1] == 60, pixels[:4]
            next_frame, changed = frame(1, sequence=first['sequence'])
            deadline=time.monotonic()+2
            while changed == pixels and time.monotonic()<deadline:
                next_frame,changed=frame(1,sequence=next_frame['sequence'])
            assert next_frame['sequence'] > first['sequence']
            assert changed != pixels, 'Canvas did not animate'
            (root/'BLUE.HTML').write_text((root/'blue.html').read_text())
            ok('create', 2, root=str(root), entry='BLUE.HTML', width=32, height=24)
            _, blue = frame(2, ready=lambda pixels: pixels[:4] == bytes([200,20,10,255]))
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
            _, green=frame(7, ready=lambda pixels: abs(pixels[1]-204) <= 2 and abs(pixels[0]-51) <= 2)
            assert abs(green[1]-204) <= 2 and abs(green[0]-51) <= 2, green[:4]
            ok('close',7)
            properties={'enabled':{'value':True},'speed':{'value':.4},'caption':{'value':'initial'}}
            ok('create',16,root=str(root),entry='callbacks.html',width=32,height=24,properties=properties)
            def callback_frame(expected, spectrum):
                deadline=time.monotonic()+5
                while time.monotonic()<deadline:
                    _,pixels=frame(16,spectrum=spectrum)
                    if pixels[:4]==bytes(expected): return
                    time.sleep(.02)
                raise AssertionError(f'Wallpaper callback pixels differ: {pixels[:4]!r}')
            callback_frame([150,50,40,255],[.25]*64+[.75]*64)
            properties['speed']['value']=.8
            properties['caption']['value']='updated'
            ok('properties',16,properties=properties)
            callback_frame([0,0,80,255],[0]*128)
            ok('pause',16)
            ok('resume',16)
            callback_frame([150,50,40,255],[.25]*64+[.75]*64)
            invalid,_=call('frame',16,width=32,height=24,sequence=0,spectrum=[0]*127)
            assert not invalid['ok'] and 'spectrum' in invalid['error']
            invalid,_=call('frame',16,width=32,height=24,sequence=0,spectrum=[-1]*128)
            assert not invalid['ok'] and 'spectrum' in invalid['error']
            frame(2)
            ok('close',16)
            ok('create',18,root=str(root),entry='sound.html',width=32,height=24)
            def audio_result(ready, ident=18):
                deadline=time.monotonic()+5
                while time.monotonic()<deadline:
                    response,_=ok('frame',ident,width=32,height=24,sequence=0)
                    if ready(response): return response
                    time.sleep(.03)
                raise AssertionError(f'Audio output did not reach expected state: {response}')
            captured=audio_result(lambda r:r['audio_packets']>5)
            assert captured['audio_peak']==0, 'Candidate audio must be muted'
            ok('audio',18,mute=False,volume=.4)
            audible=audio_result(lambda r:r['audio_peak']>.05)
            assert abs(audible['audio_peak']-.2)<.015, audible
            ok('audio',18,mute=False,volume=.1)
            time.sleep(.05)
            ok('frame',18,width=32,height=24,sequence=0)
            quiet=audio_result(lambda r:0<r['audio_peak']<.07)
            assert abs(quiet['audio_peak']-.05)<.015, quiet
            ok('audio',18,mute=True,volume=1)
            time.sleep(.05)
            ok('frame',18,width=32,height=24,sequence=0)
            assert audio_result(lambda r:True)['audio_peak']==0
            ok('pause',18)
            paused=audio_result(lambda r:True)['audio_packets']
            time.sleep(.15)
            assert audio_result(lambda r:True)['audio_packets']==paused
            ok('resume',18)
            audio_result(lambda r:r['audio_packets']>paused+5)
            ok('close',18)
            ok('create',19,root=str(root),entry='media-sound.html',width=32,height=24)
            audio_result(lambda r:r['audio_packets']>5,19)
            ok('audio',19,mute=False,volume=.4)
            media_audio=audio_result(lambda r:r['audio_peak']>.05,19)
            assert abs(media_audio['audio_peak']-.2)<.015, media_audio
            ok('close',19)
            before_crash_browser=descendants(process.pid, host)
            ok('create',17,root=str(root),entry='blue.html',width=32,height=24)
            crash,_=frame(17)
            survivor,_=frame(2)
            assert crash['renderer_client'] and crash['renderer_client']!=survivor['renderer_client'], (crash,survivor)
            fresh=descendants(process.pid, host)-before_crash_browser
            targets=set()
            for pid in fresh:
                arguments=(Path('/proc')/str(pid)/'cmdline').read_bytes().split(b'\0')
                if b'--type=zygote' in arguments or b'--type=renderer' in arguments:
                    targets.add(pid)
            # Linux's sandbox can create both an init/reaper and the renderer
            # as siblings with inherited zygote argv. Both belong to this new
            # browser; no other page is created during this interval.
            assert 1<=len(targets)<=4, (crash,fresh,targets)
            for target in targets: os.kill(target,9)
            deadline=time.monotonic()+5
            while time.monotonic()<deadline:
                crashed,_=call('frame',17,width=32,height=24,sequence=0)
                if not crashed['ok']:
                    assert 'renderer exited' in crashed['error'].lower(), crashed
                    break
                time.sleep(.02)
            else: raise AssertionError('Renderer crash was not diagnosed')
            frame(2)
            ok('close',17)
            ok('close',2)
            owned_children=descendants(process.pid, host)
            ok('shutdown')
        process.wait(timeout=10)
        assert process.returncode == 0, process.returncode
        assert all(not (Path('/proc')/str(pid)).exists() for pid in owned_children), 'CEF descendants survived shutdown'
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
