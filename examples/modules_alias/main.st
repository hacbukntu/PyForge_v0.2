// Los alias evitan colisiones. La selección importa solo los nombres indicados.
importar "math.st" como matematicas;
importar "otro.st" como otro;
desde "math.st" importar doble como duplicar;
mostrar(matematicas.doble(10));
mostrar(otro.doble(10));
mostrar(duplicar(10));
