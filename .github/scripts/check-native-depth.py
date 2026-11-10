"""Bounded fresh-process default-caller C ABI depth/reuse regression."""
import ctypes,json,os,subprocess,sys
from pathlib import Path

def child(stage,library,api,depth):
    lib=ctypes.CDLL(library);text=('('*depth+'x'+')'*depth).encode()
    if api=='model':
        parse=getattr(lib,stage+'_parse_model_json_v1');parse.argtypes=[ctypes.c_char_p,ctypes.c_size_t,ctypes.c_char_p,ctypes.c_size_t];parse.restype=ctypes.c_void_p
        free=getattr(lib,stage+'_free_json_v1')
        def call(value):return parse(value,len(value),b'.',1)
    else:
        artifact=Path('artifacts/'+{'amath':'asciimath','latexmath':'latex','umath':'umath'}[stage]+'.json').read_bytes()
        parse=lib.parsanol_parg_parse_with_compatibility;parse.argtypes=[ctypes.c_char_p]*3+[ctypes.c_uint32];parse.restype=ctypes.c_void_p
        free=lib.parsanol_parg_free
        def call(value):return parse(artifact,b'dot',value,0)
    free.argtypes=[ctypes.c_void_p];free.restype=None;rows=[]
    for label,value in [('nested',text),('reuse',b'x')]:
        pointer=call(value);assert pointer,'null native result'
        try:reply=json.loads(ctypes.string_at(pointer))
        finally:free(pointer)
        assert isinstance(reply.get('ok'),bool),reply
        if label=='reuse':assert reply['ok'],reply
        if not reply['ok']:
            category=reply.get('error',{}).get('category')
            assert category in ('recursion_limit','call_depth_limit','model_error'),reply
            if category in ('recursion_limit','call_depth_limit'):
                assert reply['error'].get('offset',reply['error'].get('byte_offset'))is None,reply
        rows.append(dict(label=label,ok=reply['ok'],error=reply.get('error')))
    print(json.dumps(dict(stage=stage,api=api,depth=depth,rows=rows)))

def main():
    if len(sys.argv)>2 and sys.argv[1]=='--child':child(sys.argv[2],sys.argv[3],sys.argv[4],int(sys.argv[5]));return
    stage=sys.argv[1];filename=stage+'.dll'if sys.platform=='win32'else'lib'+stage+('.dylib'if sys.platform=='darwin'else'.so');target=Path(os.environ.get('CARGO_TARGET_DIR','target'));library=(target/'debug'/filename).resolve();rows=[]
    for api in ['raw','model']:
        for depth in [64,128,256]:
            result=subprocess.run([sys.executable,str(Path(__file__).resolve()),'--child',stage,str(library),api,str(depth)],capture_output=True,text=True,timeout=120)
            assert result.returncode==0,dict(api=api,depth=depth,exit=result.returncode,stderr=result.stderr,stdout=result.stdout)
            rows.append(json.loads(result.stdout))
    evidence=Path('target/native-depth-proof.json');evidence.parent.mkdir(exist_ok=True);evidence.write_text(json.dumps(rows,indent=2)+'\n');print(json.dumps(rows))
if __name__=='__main__':main()
