// Cada variable conserva su tipo. cadena usa texto UTF-8 y flotante usa f64.
variable nombre: cadena = "Ana";
variable edad: entero = 20;
variable precio: flotante = 12.5;
variable activo: booleano = verdadero;
mostrar(nombre);
mostrar(edad);
mostrar(precio);
mostrar(activo);

// Concatenar crea otra cadena; no modifica las referencias anteriores.
funcion saludar(persona: cadena) -> cadena {
    retornar "Hola, " + persona;
}
mostrar(saludar(nombre));
nombre += " 😀";
mostrar(nombre);
mostrar(longitud(nombre));
mostrar(precio * 2.0);

clase Persona {
    variable nombre: cadena;
    variable edad: entero;
}

implementar Persona {
    funcion presentarse() -> cadena {
        retornar "Me llamo " + yo.nombre + " y tengo " + cadena(yo.edad) + " años.";
    }
}

