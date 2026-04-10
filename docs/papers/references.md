# References

Primary academic sources for the ontologies in this repository.

## Geometry

- Hilbert, D. (1899). *Grundlagen der Geometrie*. Teubner, Leipzig. [The axiomatic foundation of Euclidean geometry — 20 axioms in 5 groups: incidence, order, congruence, parallelism, continuity.]
- Avigad, J., Dean, E., & Mumma, J. (2009). "A Formal System for Euclid's Elements." *Review of Symbolic Logic*, 2(4):700-768. [PDF](https://www.andrew.cmu.edu/user/avigad/Papers/formal_system_for_euclids_elements.pdf)
- Coxeter, H.S.M. (1969). *Introduction to Geometry* (2nd ed.). Wiley.

## Rotation (SO(3)) and Rigid Motion (SE(3))

- Shuster, M.D. (1993). "A Survey of Attitude Representations." *Journal of the Astronautical Sciences*, 41(4):439-517. [Comprehensive comparison of rotation representations: quaternion, DCM, Euler, axis-angle, MRP.]
- Sola, J. (2017). "Quaternion kinematics for the error-state Kalman filter." arXiv:1711.02508. [Quaternion conventions, composition, perturbation, for sensor fusion.]
- Murray, R.M., Li, Z., & Sastry, S.S. (1994). *A Mathematical Introduction to Robotic Manipulation*. CRC Press. [SE(3) Lie group, twists, wrenches, exponential coordinates.]
- Kumar, V. "Rigid Body Kinematics and the Lie group SE(3)." University of Pennsylvania MEAM 620 lecture notes. [PDF](https://www.seas.upenn.edu/~meam620/slides/kinematicsI.pdf)
- Kim, J. "Lie Group Formulation of Articulated Rigid Body Dynamics." CMU Technical Report. [PDF](https://www.cs.cmu.edu/~junggon/tools/liegroupdynamics.pdf)

## Time

- Allen, J.F. (1983). "Maintaining Knowledge about Temporal Intervals." *Communications of the ACM*, 26(11):832-843. [The 13 interval relations — foundational temporal reasoning.]
- Grüninger, M. & Li, Z. (2017). "The Time Ontology of Allen's Interval Algebra." *TIME 2017*, LIPIcs Vol. 90. [PDF](https://drops.dagstuhl.de/storage/00lipics/lipics-vol090-time2017/LIPIcs.TIME.2017.16/LIPIcs.TIME.2017.16.pdf)
- W3C (2017). "Time Ontology in OWL." W3C Recommendation. [Specification](https://www.w3.org/TR/owl-time/)
- Allan, D.W. (1966). "Statistics of Atomic Frequency Standards." *Proceedings of the IEEE*, 54(2):221-230.
- Riley, W.J. (2008). *Handbook of Frequency Stability Analysis*. NIST Special Publication 1065. [PDF](https://tf.nist.gov/general/pdf/2220.pdf)
- IEEE Std 1139-2008. "Standard Definitions of Physical Quantities for Fundamental Frequency and Time Metrology — Random Instabilities."

## Time Systems

- IAU 2000 Resolution B1.9: Definition of Terrestrial Time (TT = TAI + 32.184 s).
- IAU 2006 Resolution B3: Barycentric Coordinate Time (TCB) and related scales.
- IS-GPS-200 (2022). "Interface Specification: Navstar GPS Space Segment / Navigation User Segment Interfaces." US Space Force. [GPS time definition: GPS = TAI - 19 s.]
- ITU-R TF.460 (2002). "Standard-frequency and time-signal emissions." [UTC definition with leap seconds.]
- Ashby, N. (2003). "Relativity in the Global Positioning System." *Living Reviews in Relativity*, 6(1). [PMC](https://pmc.ncbi.nlm.nih.gov/articles/PMC5253894/)
- ESA Navipedia. "Transformations between Time Systems." [Reference](https://gssc.esa.int/navipedia/index.php/Transformations_between_Time_Systems)

## Linear Algebra

- Axler, S. (2024). *Linear Algebra Done Right* (4th ed.). Springer. [Vector space axioms, eigenvalues, spectral theorem.]
- Strang, G. (2023). *Introduction to Linear Algebra* (6th ed.). Wellesley-Cambridge Press. [Positive definite matrices, Cholesky, SVD.]
- Kahan, W. "Axioms for Fields and Vector Spaces." UC Berkeley Math H110 notes. [PDF](https://people.eecs.berkeley.edu/~wkahan/MathH110/Axioms.pdf)
- Horn, R.A. & Johnson, C.R. (2013). *Matrix Analysis* (2nd ed.). Cambridge University Press. [Determinant properties, matrix inequalities, Schur complement.]

## Probability and Estimation Theory

- Kolmogorov, A.N. (1933). *Grundbegriffe der Wahrscheinlichkeitsrechnung*. Springer, Berlin. [The axiomatic foundation of probability: 3 axioms.] [Archive](https://archive.org/details/kolmogorov_202112)
- Tao, T. (2015). "275A, Notes 0: Foundations of probability theory." [Blog](https://terrytao.wordpress.com/2015/09/29/275a-notes-0-foundations-of-probability-theory/)
- Mahalanobis, P.C. (1936). "On the generalized distance in statistics." *Proceedings of the National Institute of Sciences of India*, 2(1):49-55.
- Fisher, R.A. (1925). "Theory of Statistical Estimation." *Mathematical Proceedings of the Cambridge Philosophical Society*, 22(5):700-725. [Fisher information, Cramér-Rao bound.]

## Kinematics

- Goldstein, H., Poole, C., & Safko, J. (2002). *Classical Mechanics* (3rd ed.). Addison-Wesley. [Lagrangian and Hamiltonian formulations, rigid body dynamics.]
- Shabana, A.A. (2020). *Dynamics of Multibody Systems* (5th ed.). Cambridge University Press. [Kinematics on manifolds, screw theory.]
- Bernstein, D.S., Goel, A., & Ansari, A. "Geometry, Kinematics, Statics, and Dynamics." Cornell University. [PDF](http://ruina.tam.cornell.edu/Courses/ME4730%20Fall%202018/books/Dynamics_Book_Bernstein__Goel__Ansari_V02.pdf)

## Geodesy

- Torge, W. & Müller, J. (2012). *Geodesy* (4th ed.). de Gruyter. [WGS84, ellipsoid, coordinate systems.]
- NIMA (2000). "Department of Defense World Geodetic System 1984." Technical Report TR8350.2 (3rd ed.). [WGS84 specification: a = 6378137.0 m, f = 1/298.257223563.]
- Bowring, B.R. (1976). "Transformation from spatial to geographical coordinates." *Survey Review*, 23(181):323-327. [Geodetic ↔ ECEF conversion algorithm.]

## Sensor Fusion

- Kalman, R.E. (1960). "A New Approach to Linear Filtering and Prediction Problems." *Journal of Basic Engineering*, 82(1):35-45. [The Kalman filter.]
- Bar-Shalom, Y., Li, X.R., & Kirubarajan, T. (2001). *Estimation with Applications to Tracking and Navigation*. Wiley. [Multi-target tracking, data association, JPDA, MHT.]
- Groves, P.D. (2013). *Principles of GNSS, Inertial, and Multisensor Integrated Navigation Systems* (2nd ed.). Artech House. [INS/GNSS integration, strapdown mechanization, lever arm, boresight.]
- Maybeck, P.S. (1979). *Stochastic Models, Estimation, and Control* (Vols. 1-3). Academic Press. [State space models, Kalman filter theory.]
- Thrun, S., Burgard, W., & Fox, D. (2005). *Probabilistic Robotics*. MIT Press. [SLAM, particle filters, occupancy grids.]

## Military Standards

- US DoD JDL (1999). "Data Fusion Lexicon." Joint Directors of Laboratories, Data Fusion Sub-Panel. [JDL fusion model: Levels 0-5.]
- STANAG 4586 (2012). "Standard Interfaces of UAV Control System for NATO UAV Interoperability." NATO.
- MISB ST 0601 (2021). "UAS Datalink Local Set." Motion Imagery Standards Board. [Sensor metadata for UAS platforms.]
- MIL-STD-1553B (1978). "Aircraft Internal Time Division Command/Response Multiplex Data Bus." US DoD.

## Clock Characterization

- IEEE Std 1139-2008. "Standard Definitions of Physical Quantities for Fundamental Frequency and Time Metrology."
- ITU-R TF.538-4 (2017). "Measures for random instabilities in frequency and time." [Recommendation](https://www.itu.int/dms_pubrec/itu-r/rec/tf/R-REC-TF.538-4-201707-I!!PDF-E.pdf)
- Rohde & Schwarz (2019). "Time Domain Oscillator Stability Measurements." Application Note 1EF69. [PDF](https://scdn.rohde-schwarz.com/ur/pws/dl_downloads/dl_application/application_notes/1ef69/1EF69_4e_TD_Osc_Stability_Allan.pdf)

## Signal Processing

- Shannon, C.E. (1949). "Communication in the Presence of Noise." *Proceedings of the IRE*, 37(1):10-21.
- Nyquist, H. (1928). "Certain Topics in Telegraph Transmission Theory." *Transactions of the AIEE*, 47(2):617-644.
- Oppenheim, A.V. & Willsky, A.S. (1997). *Signals and Systems* (2nd ed.). Prentice Hall.
- Peyré, G. (2019). *Mathematical Foundations of Data Sciences*. CNRS/DMA. [PDF](https://mathematical-tours.github.io/book-sources/chapters-pdf/shannon.pdf)
- Byrne, C.L. *Mathematics of Signal Processing: A First Course*. UMass Lowell. [PDF](https://faculty.uml.edu/cbyrne/sp1text.pdf)

## Statistics

- Fisher, R.A. (1925). "Theory of Statistical Estimation." *Mathematical Proceedings of the Cambridge Philosophical Society*, 22(5):700-725.
- Neyman, J. & Pearson, E.S. (1933). "On the Problem of the Most Efficient Tests of Statistical Hypotheses." *Philosophical Transactions A*, 231:289-337.
- Student (Gosset, W.S.) (1908). "The Probable Error of a Mean." *Biometrika*, 6(1):1-25.
- Cramér, H. (1946). *Mathematical Methods of Statistics*. Princeton University Press.
- Rao, C.R. (1945). "Information and the accuracy attainable in the estimation of statistical parameters." *Bulletin of the Calcutta Mathematical Society*, 37:81-91.

## Control Theory

- Åström, K.J. & Murray, R.M. (2008). *Feedback Systems: An Introduction for Scientists and Engineers*. Princeton University Press. [PDF](http://www.cds.caltech.edu/~murray/books/AM08/pdf/fbs-public_24Jul2020.pdf)
- Ogata, K. (2010). *Modern Control Engineering* (5th ed.). Prentice Hall.
- Lyapunov, A.M. (1892). "The General Problem of the Stability of Motion." Kharkov Mathematical Society.
- Ziegler, J.G. & Nichols, N.B. (1942). "Optimum Settings for Automatic Controllers." *Transactions of the ASME*, 64:759-768.
- Doyle, J.C., Francis, B.A. & Tannenbaum, A.R. (1992). *Feedback Control Theory*. Macmillan.
