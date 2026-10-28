importar "cifrado";
// Clave efímera: no se imprime ni se guarda. Perder una clave impide recuperar sus datos.
variable clave = cifrado.clave();
variable sobre = cifrado.cifrar("Texto confidencial de prueba", clave, "ejemplo-ST");
//mostrar(cifrado.descifrar(sobre, clave, "ejemplo-ST"));
// Cambiar el contexto autenticado hace fallar la recuperación.
intentar { 
    mostrar(cifrado.descifrar(sobre, clave, "ejemplo-STa")); 
    }
capturar error { 
    mostrar(error); 
    }
