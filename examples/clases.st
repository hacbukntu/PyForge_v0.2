// El constructor recibe todos los campos en el orden de declaración.
clase Persona {
    variable nombre: cadena;
    variable edad: entero;

    // yo es la referencia a la instancia sobre la que se llama el método.
    funcion saludar() -> cadena {
        retornar "Hola, soy " + yo.nombre;
    }
}

// Los métodos también se pueden declarar en un bloque separado del mismo archivo.
implementar Persona {
    funcion cumplir_anios() {
        yo.edad = yo.edad + 1;
    }
}

variable persona: Persona = nuevo Persona("Ana", 20);
persona.cumplir_anios();
mostrar(persona.saludar());
mostrar(persona.edad);
