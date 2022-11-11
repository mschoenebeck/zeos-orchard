const zeos = import('./pkg/zeos_orchard.js');

zeos.then(m => {
    console.log(m.test1(null));
}).catch(console.error);
