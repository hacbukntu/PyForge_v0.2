// Ejemplo final para validar colores en VS Code
// Variables, funciones, clases, propiedades, cadenas, números y operadores.

variable nombre: cadena = "Ana";
variable edad: entero = 20;
variable activo: booleano = verdadero;
variable total: entero = 10 + 5 * 2;

funcion saludar(persona: cadena) -> cadena {
    retornar "Hola, " + persona + "!";
}

funcion sumar(a: entero, b: entero) -> entero {
    retornar a + b;
}

clase Persona {
    variable nombre: cadena;
    variable edad: entero;

    funcion saludar() -> cadena {
        retornar "Hola, soy " + yo.nombre;
    }

    funcion cumplir_anios() {
        yo.edad = yo.edad + 1;
    }
}

implementar Persona {
    funcion mostrar_edad() {
        mostrar("Edad: " + yo.edad);
    }
}

variable persona: Persona = nuevo Persona("Ana", 20);
mostrar(saludar(nombre));
mostrar(sumar(5, 7));
mostrar(persona.saludar());
persona.cumplir_anios();
mostrar(persona.edad);
mostrar(activo);
mostrar(total);
