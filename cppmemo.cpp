#include <iostream>
#include <vector>
#include <string>
#include <cmath>
#include <algorithm>
#include <set>

//乱数
// 注意: longではなくint(32bit)にすべき
unsigned long xor128(){
  static unsigned long x=123456789,y=362436069,z=521288629,w=88675123;
  unsigned long t;
  t=(x^(x<<11));x=y;y=z;z=w; return( w=(w^(w>>19))^(t^(t>>8)) );
}

//maximal cardinalities in the range [0..10^9] and for common "practical" values m=2^4,...,2^16
#define UINT unsigned int
class HyperLogLog {
  int b, m; //register-index-width and register-size

  std::set<std::string> s; //for real-value

  //registers
  std::vector<int> M;

  //alpha
  double alpha(int mm){
    if(mm == 16) return 0.673;
    if(mm == 32) return 0.697;
    if(mm == 64) return 0.709;
    if(mm == 128 ||
              mm == 256 ||
              mm == 512 ||
              mm == 1024 ||
              mm == 2048 ||
              mm == 4096 ||
              mm == 8192 ||
              mm == 16384 ||
              mm == 32768 ||
       mm == 65536){
      return 0.7213 / (1.0 + 1.079/mm);
    }
    return 0.0;
  }

  //h : D -> {0,1}^32 hash( not good? :( )
  UINT hash(const std::string& str){
    UINT ret = 0;
    for(int i=0; i<str.length(); i++){
      ret = ret * 123456789 + str[i];
    }
    return ret;
  }

  //bit operations
  UINT lower_bit(UINT x){
    int mask = (1 << b) - 1;
    return x & mask;
  }
  UINT upper_bit(UINT x){
    return x >> b;
  }
  int rho(UINT x){
    for(int i=1; i<=32; i++){
      if(x & 1) return i;
      x >>= 1;
    }
    return 33;
  }

  //harmonic means of all M[j]
  double harmonic_mean(){
    double ret = 0.0;
    for(int j=1; j<=m; j++){
      ret += 1.0/pow(2.0, M[j]);
    }
    return 1.0/ret;
  }

public:
  // assume m = 2^b, b in [4,16]
  HyperLogLog(int _m):m(_m), M(_m+1,0){
    //find b from m
    b = -1;
    for(int p=16, i=4; i<=16; i++){
      if(p == m){
	b = i;
	break;
      }
      p *= 2;
    }
  }

  //count up
  void add(const std::string& str){
    s.insert(str);
    UINT x = hash(str);
    int j = 1 + lower_bit(x);
    int w = upper_bit(x);
    M[j] = std::max(M[j], rho(w));
  }

  //estimate the number of distinct elements(the cardinality)
  int estimate(){
    double E = alpha(m) * m * m * harmonic_mean();
    if(E <= 2.5 * m){
      int V = 0;
      for(int i=1; i<M.size(); i++) if(M[i] == 0) V++;
      if(V != 0) E = (double)m * log((double)m/V);
    }
    if(E <= 1.0/30.0 * 4294967296.0){
      ;
    }else{
      E = -4294967296.0 * log2(1.0 - E/4294967296.0);
    }

    //real-value   estimate-value  error(%)
    std::cout << s.size() << "\t" << E << "\t" << (E - s.size())/s.size()*100.0 << std::endl;

    return (int)E; //cardinality estimate E* with typical relative error +- 1.04/sqrt(m)
  }

  //dump register values
  void dump_register_value(){
    for(int j=1; j<=m; j++){
      std::cout << M[j];
      if(j!=m) std::cout << " ";
      else std::cout << std::endl;
    }
  }

};



//ランダムな文字列(n文字のアルファベット文字列)
std::string random_string(int n){
  std::string ret = "";
  for(int i=0; i<n; i++){
    ret += 'a' + xor128()%26;
  }
  return ret;
}

int main(){
  HyperLogLog hll(64); //レジスタは64個

  for(int i=0; i<100000; i++){
    hll.add(random_string(4));
    //hll.dump_register_value();
    hll.estimate();
  }

  return 0;
}
