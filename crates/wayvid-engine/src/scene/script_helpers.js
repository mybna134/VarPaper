// Value adapters only; VM lifetime, execution, timers, audio and rendering
// are owned by the Rust host. No filesystem or process API is installed.
class Vec2 {
    constructor(x=0,y=x) { if(typeof x==='object'){this.x=x.x;this.y=x.y;}else{this.x=x;this.y=y;} }
    add(v){return new Vec2(this.x+v.x,this.y+v.y);}
    subtract(v){return new Vec2(this.x-v.x,this.y-v.y);}
    multiply(v){return typeof v==='number'?new Vec2(this.x*v,this.y*v):new Vec2(this.x*v.x,this.y*v.y);}
    toJSON(){return [this.x,this.y];}
}
class Vec3 extends Vec2 {
    constructor(x=0,y=x,z=x){super(x,y);this.z=typeof x==='object'?x.z:z;}
    add(v){return new Vec3(this.x+v.x,this.y+v.y,this.z+v.z);}
    subtract(v){return new Vec3(this.x-v.x,this.y-v.y,this.z-v.z);}
    multiply(v){return typeof v==='number'?new Vec3(this.x*v,this.y*v,this.z*v):new Vec3(this.x*v.x,this.y*v.y,this.z*v.z);}
    toJSON(){return [this.x,this.y,this.z];}
}
class Vec4 extends Vec3 {
    constructor(x=0,y=x,z=x,w=x){super(x,y,z);this.w=typeof x==='object'?x.w:w;}
    toJSON(){return [this.x,this.y,this.z,this.w];}
}
Object.assign(globalThis,{Vec2,Vec3,Vec4});
for (const layer of Object.values(__sceneLayers)) {
    for(const key of ['origin','scale','angles','size','color']) {
        let v=layer[key];
        if(typeof v==='string')v=v.trim().split(/\s+/).map(Number);
        if(Array.isArray(v))layer[key]=v.length===2?new Vec2(...v):v.length===3?new Vec3(...v):new Vec4(...v);
    }
}
globalThis.thisScene={time:0,currentTime:0,dt:0,fps:60,getLayer:function(name){
    for(const layer of Object.values(__sceneLayers))if(layer.name===name||String(layer.id)===String(name))return layer;
    return null;
}};
globalThis.__createScriptProperties=function(seed){
    const values=Object.assign({},seed||{});
    const builder={};
    const add=function(option){
        if(!option||typeof option.name!=='string')throw new TypeError('Invalid script property');
        if(Object.keys(values).length>=256)throw new RangeError('Script property budget exceeded');
        if(!(option.name in values))values[option.name]=option.value;return builder;
    };
    for(const name of ['addSlider','addCheckbox','addCombo','addColor','addText'])builder[name]=add;
    builder.finish=function(){return values;};
    return builder;
};

globalThis.createScriptProperties=function(){return __createScriptProperties({});};
